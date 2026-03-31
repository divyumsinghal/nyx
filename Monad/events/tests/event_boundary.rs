use nun::NyxApp;
use nyx_events::{
    subjects, DomainEvent, EventEnvelope, EventPublisher, InMemoryEventPublisher,
    NoopEventPublisher,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct TestDomainEvent {
    alias: String,
}

impl DomainEvent for TestDomainEvent {
    const APP: NyxApp = NyxApp::Uzume;
    const SUBJECT: &'static str = subjects::UZUME_PROFILE_UPDATED;
}

#[test]
fn domain_event_envelope_builds_without_provider_metadata() {
    // #given a domain event payload
    let event = TestDomainEvent {
        alias: "owner_alias".to_string(),
    };

    // #when creating a provider-agnostic event envelope
    let envelope = EventEnvelope::from_domain(&event).unwrap();

    // #then envelope carries app + subject + payload only
    assert_eq!(envelope.app, NyxApp::Uzume);
    assert_eq!(envelope.subject, subjects::UZUME_PROFILE_UPDATED);
    assert_eq!(envelope.payload["alias"], "owner_alias");
}

#[tokio::test]
async fn in_memory_adapter_is_deterministic() {
    // #given in-memory event publisher
    let publisher = InMemoryEventPublisher::default();
    let first = EventEnvelope::new(
        NyxApp::Uzume,
        subjects::UZUME_POST_CREATED,
        serde_json::json!({ "post_id": "1" }),
    );
    let second = EventEnvelope::new(
        NyxApp::Uzume,
        subjects::UZUME_POST_DELETED,
        serde_json::json!({ "post_id": "1" }),
    );

    // #when publishing events
    publisher.publish(first.clone()).await.unwrap();
    publisher.publish(second.clone()).await.unwrap();

    // #then snapshot preserves deterministic append order
    let snapshot = publisher.snapshot();
    assert_eq!(snapshot.len(), 2);
    assert_eq!(snapshot[0], first);
    assert_eq!(snapshot[1], second);
}

#[tokio::test]
async fn noop_adapter_accepts_events() {
    // #given noop event publisher
    let publisher = NoopEventPublisher;

    // #when publishing an event
    let result = publisher
        .publish(EventEnvelope::new(
            NyxApp::Uzume,
            subjects::UZUME_POST_CREATED,
            serde_json::json!({ "post_id": "2" }),
        ))
        .await;

    // #then call succeeds without side effects
    assert!(result.is_ok());
}
