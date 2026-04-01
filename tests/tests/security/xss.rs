use nyx_tests::security::xss;

#[test]
fn xss_corpus_contains_script_and_event_handler_payloads() {
    let payloads = xss::all_payloads();
    assert!(payloads.iter().any(|p| p.contains("<script")));
    assert!(payloads
        .iter()
        .any(|p| p.contains("onerror") || p.contains("onload")));
}

#[test]
fn xss_corpus_contains_protocol_and_markdown_vectors() {
    let payloads = xss::all_payloads();
    assert!(payloads.iter().any(|p| p.contains("javascript:")));
    assert!(payloads.iter().any(|p| p.contains("[Click](javascript:")));
}
