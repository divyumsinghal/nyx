use std::collections::HashSet;
use std::path::PathBuf;

use events::subjects::{
    MediaProcessedPayload, MediaUploadedPayload, UZUME_MEDIA_PROCESSED, UZUME_MEDIA_UPLOADED,
};
use events::{NatsClient, Publisher, Subscriber};
use futures::StreamExt;
use oya::config::ProcessingConfig;
use oya::pipeline::{MediaJob, MediaPipeline, PipelineResult};
use tokio::signal;
use tracing::{error, info, warn};

struct Worker {
    pipeline: MediaPipeline,
    publisher: Publisher,
    processed_jobs: HashSet<uuid::Uuid>,
    temp_dir: PathBuf,
}

impl Worker {
    fn new(pipeline: MediaPipeline, publisher: Publisher, temp_dir: PathBuf) -> Self {
        Self {
            pipeline,
            publisher,
            processed_jobs: HashSet::new(),
            temp_dir,
        }
    }

    async fn handle_event(&mut self, payload: MediaUploadedPayload) {
        if self.processed_jobs.contains(&payload.job_id) {
            info!(job_id = %payload.job_id, "duplicate event, skipping");
            return;
        }

        info!(
            job_id = %payload.job_id,
            entity_type = %payload.entity_type,
            entity_id = %payload.entity_id,
            "processing media upload"
        );

        let result = self.process_media(&payload).await;

        match result {
            Ok(pipeline_result) => {
                self.processed_jobs.insert(payload.job_id);
                if let Err(e) = self.emit_processed_event(&pipeline_result).await {
                    error!(error = %e, "failed to emit processed event");
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
        let is_video = payload.mime_type.starts_with("video/");
        let job = MediaJob {
            job_id: payload.job_id,
            entity_type: &payload.entity_type,
            entity_id: &payload.entity_id,
            mime_type: &payload.mime_type,
        };

        if is_video {
            let input_path = PathBuf::from(&payload.source_path);
            let output_dir = self.temp_dir.join(&payload.entity_id);

            let result = self.pipeline.process_video(job, &input_path, &output_dir)?;

            Ok(PipelineResult::Video(result))
        } else {
            let data = tokio::fs::read(&payload.source_path).await?;

            let result = self.pipeline.process_image(job, &data)?;

            Ok(PipelineResult::Image(result))
        }
    }

    async fn emit_processed_event(
        &self,
        result: &PipelineResult,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut variants = std::collections::HashMap::new();

        match result {
            PipelineResult::Image(img_result) => {
                for v in &img_result.variants {
                    variants.insert(
                        v.name.clone(),
                        format!(
                            "Uzume/{}/{}/{}.{}",
                            img_result.entity_type,
                            img_result.entity_id,
                            v.name,
                            match v.mime_type.as_str() {
                                "image/jpeg" => "jpg",
                                "image/png" => "png",
                                "image/webp" => "webp",
                                _ => "bin",
                            }
                        ),
                    );
                }
            }
            PipelineResult::Video(vid_result) => {
                for v in &vid_result.video_result.variants {
                    variants.insert(
                        v.name.clone(),
                        format!(
                            "Uzume/{}/{}/hls/{}/",
                            vid_result.entity_type, vid_result.entity_id, v.name
                        ),
                    );
                }
                variants.insert(
                    "poster".to_string(),
                    format!(
                        "Uzume/{}/{}/poster.jpg",
                        vid_result.entity_type, vid_result.entity_id
                    ),
                );
                variants.insert(
                    "master".to_string(),
                    format!(
                        "Uzume/{}/{}/hls/master.m3u8",
                        vid_result.entity_type, vid_result.entity_id
                    ),
                );
            }
        }

        let processed_payload = MediaProcessedPayload {
            job_id: result.job_id(),
            entity_type: result.entity_type().to_string(),
            entity_id: result.entity_id().to_string(),
            variants,
            processing_ms: result.processing_ms(),
        };

        self.publisher
            .publish(UZUME_MEDIA_PROCESSED, processed_payload)
            .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("oya_worker=info".parse()?),
        )
        .init();

    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into());
    let temp_dir = std::env::var("OYA_TEMP_DIR").unwrap_or_else(|_| "/tmp/oya".into());

    info!(nats_url = %nats_url, "connecting to NATS");

    let client = NatsClient::connect(&nats_url).await?;

    client
        .ensure_stream("nyx-events", vec![UZUME_MEDIA_UPLOADED.to_string()])
        .await?;

    let config = ProcessingConfig::default();
    let pipeline = MediaPipeline::new(config.clone());
    let publisher = Publisher::new(client.clone(), "oya");
    let subscriber = Subscriber::new(client);

    let mut worker = Worker::new(pipeline, publisher, PathBuf::from(temp_dir));

    info!("subscribing to {}", UZUME_MEDIA_UPLOADED);
    let mut event_stream = subscriber
        .subscribe::<MediaUploadedPayload>(UZUME_MEDIA_UPLOADED)
        .await?;

    info!("oya-worker ready, waiting for media upload events");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("received shutdown signal");
        }
        _ = async {
            while let Some(event_result) = event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        info!(
                            event_id = %event.id,
                            job_id = %event.payload.job_id,
                            "received media uploaded event"
                        );
                        worker.handle_event(event.payload).await;
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to parse event");
                    }
                }
            }
        } => {}
    }

    info!("oya-worker shutting down");
    Ok(())
}
