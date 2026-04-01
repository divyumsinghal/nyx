//! NATS setup tests — unit portion tests pure config logic with no network.

use nyx_xtask::commands::nats_setup::{nyx_stream_config, uzume_stream_config};

#[test]
fn nyx_stream_name_is_correct() {
    let cfg = nyx_stream_config();
    assert_eq!(cfg.name, "NYX");
}

#[test]
fn nyx_stream_subjects_include_wildcard() {
    let cfg = nyx_stream_config();
    assert!(cfg.subjects.iter().any(|s| s == "nyx.>"));
}

#[test]
fn uzume_stream_name_is_correct() {
    let cfg = uzume_stream_config();
    assert_eq!(cfg.name, "UZUME");
}

#[test]
fn uzume_stream_subjects_include_wildcard() {
    let cfg = uzume_stream_config();
    assert!(cfg.subjects.iter().any(|s| s == "Uzume.>"));
}

#[test]
fn nyx_stream_has_exactly_one_subject() {
    let cfg = nyx_stream_config();
    assert_eq!(cfg.subjects.len(), 1);
}

#[test]
fn uzume_stream_has_exactly_one_subject() {
    let cfg = uzume_stream_config();
    assert_eq!(cfg.subjects.len(), 1);
}
