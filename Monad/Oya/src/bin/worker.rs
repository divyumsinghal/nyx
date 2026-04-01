//! # oya-worker — NATS media processing subscriber
//!
//! Subscribes to `Uzume.media.uploaded` and dispatches each job through the
//! [`MediaPipeline`](oya::MediaPipeline). On success publishes a
//! `Uzume.media.processed` event with the generated variant paths.
//!
//! ## Environment variables
//!
//! | Variable         | Default                  | Description                        |
//! |------------------|--------------------------|------------------------------------|
//! | `NATS_URL`       | `nats://localhost:4222`  | NATS JetStream server URL          |
//! | `OYA_TEMP_DIR`   | `/tmp/oya`               | Temporary directory for video work |
//! | `RUST_LOG`       | —                        | Log filter (e.g. `oya_worker=info`)|

use std::collections::HashSet;
use std::path::PathBuf;

use futures::StreamExt;
use oya::events::{
    MediaProcessedPayload, MediaUploadedPayload, NyxEvent, UZUME_MEDIA_PROCESSED,
    UZUME_MEDIA_UPLOADED,
};
use oya::pipeline::{MediaJob, MediaPipeline, PipelineResult};
use oya::ProcessingConfig;
use tokio::signal;
use tracing::{error, info, warn};

struct Worker {
    pipeline: MediaPipeline,
    nats: async_nats::Client,
    processed_jobs: HashSet<uuid::Uuid>,
    temp_dir: PathBuf,
}

impl Worker {
    fn new(pipeline: MediaPipeline, nats: async_nats::Client, temp_dir: PathBuf) -> Self {
        Self {
            pipeline,
            nats,
            processed_jobs: HashSet::new(),
            temp_dir,
        }
    }

    async fn handle_event(&mut self, payload: MediaUploadedPayload) {
        if self.processed_jobs.contains(&payload.job_id) {
            info!(job_id = %payload.job_id, "duplicate job, skipping");
            return;
        }

        info!(
            job_id = %payload.job_id,
            entity_type = %payload.entity_type,
            entity_id = %payload.entity_id,
            mime_type = %payload.mime_type,
            "processing media upload"
        );

        match self.process_media(&payload).await {
            Ok(result) => {
                self.processed_jobs.insert(payload.job_id);
                if let Err(e) = self.emit_processed(&result).await {
                    error!(error = %e, job_id = %payload.job_id, "failed to emit processed event");
                }
            }
            Err(e) => {
                error!(
                    job_id = %payload.job_id,
                    error = %e,
                    "media processing failed"
                );
            }
        }
    }

    async fn process_media(
        &self,
        payload: &MediaUploadedPayload,
    ) -> Result<PipelineResult, Box<dyn std::error::Error + Send + Sync>> {
        let job = MediaJob {
            job_id: payload.job_id,
            entity_type: &payload.entity_type,
            entity_id: &payload.entity_id,
            mime_type: &payload.mime_type,
        };

        if payload.mime_type.starts_with("video/") {
            let input_path = PathBuf::from(&payload.source_path);
            let output_dir = self.temp_dir.join(&payload.entity_id);
            let result = self.pipeline.process_video(job, &input_path, &output_dir)?;
            Ok(PipelineResult::Video(result))
        } else {
            // Image: download from local path (worker is expected to have the
            // raw file available, e.g. via a shared volume or pre-download step).
            let data = tokio::fs::read(&payload.source_path).await?;
            let result = self.pipeline.process_image(job, &data)?;
            Ok(PipelineResult::Image(result))
        }
    }

    async fn emit_processed(
        &self,
        result: &PipelineResult,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut variants = std::collections::HashMap::new();

        match result {
            PipelineResult::Image(img) => {
                for v in &img.variants {
                    let ext = match v.mime_type.as_str() {
                        "image/jpeg" => "jpg",
                        "image/png" => "png",
                        "image/webp" => "webp",
                        _ => "bin",
                    };
                    variants.insert(
                        v.name.clone(),
                        format!("Uzume/{}/{}/{}.{}", img.entity_type, img.entity_id, v.name, ext),
                    );
                }
            }
            PipelineResult::Video(vid) => {
                for v in &vid.video_result.variants {
                    variants.insert(
                        v.name.clone(),
                        format!(
                            "Uzume/{}/{}/hls/{}/",
                            vid.entity_type, vid.entity_id, v.name
                        ),
                    );
                }
                variants.insert(
                    "poster".to_string(),
                    format!("Uzume/{}/{}/poster.jpg", vid.entity_type, vid.entity_id),
                );
                variants.insert(
                    "master".to_string(),
                    format!(
                        "Uzume/{}/{}/hls/master.m3u8",
                        vid.entity_type, vid.entity_id
                    ),
                );
            }
        }

        let processed = MediaProcessedPayload {
            job_id: result.job_id(),
            entity_type: result.entity_type().to_string(),
            entity_id: result.entity_id().to_string(),
            variants,
            processing_ms: result.processing_ms(),
        };

        let event = NyxEvent::new(UZUME_MEDIA_PROCESSED, "oya", processed);
        let json = serde_json::to_vec(&event)?;
        self.nats
            .publish(UZUME_MEDIA_PROCESSED, json.into())
            .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("oya_worker=info".parse()?),
        )
        .init();

    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let temp_dir = std::env::var("OYA_TEMP_DIR").unwrap_or_else(|_| "/tmp/oya".to_string());

    info!(nats_url = %nats_url, "connecting to NATS");
    let nats = async_nats::connect(&nats_url).await?;

    let config = ProcessingConfig::default();
    let pipeline = MediaPipeline::new(config);

    let mut worker = Worker::new(pipeline, nats.clone(), PathBuf::from(temp_dir));

    info!(subject = UZUME_MEDIA_UPLOADED, "subscribing to NATS subject");
    let mut subscription = nats.subscribe(UZUME_MEDIA_UPLOADED).await?;

    info!("oya-worker ready, waiting for media upload events");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("received shutdown signal");
        }
        _ = async {
            while let Some(msg) = subscription.next().await {
                match serde_json::from_slice::<NyxEvent<MediaUploadedPayload>>(&msg.payload) {
                    Ok(event) => {
                        info!(
                            event_id = %event.id,
                            job_id = %event.payload.job_id,
                            "received media.uploaded event"
                        );
                        worker.handle_event(event.payload).await;
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to parse event payload, skipping");
                    }
                }
            }
        } => {}
    }

    info!("oya-worker shutting down");
    Ok(())
}
