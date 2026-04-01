use nyx_tests::security::{edge_cases, sql_injection};

#[test]
fn sql_injection_corpus_contains_high_risk_vectors() {
    let payloads = sql_injection::all_payloads();
    assert!(payloads.iter().any(|p| p.contains("UNION")));
    assert!(payloads.iter().any(|p| p.contains("DROP TABLE")));
    assert!(payloads.iter().any(|p| p.contains("SLEEP")));
}

#[test]
fn sql_injection_corpus_rejects_empty_values() {
    let payloads = sql_injection::all_payloads();
    assert!(payloads.iter().all(|p| !p.trim().is_empty()));
}

#[test]
fn sql_injection_related_edge_cases_include_null_byte() {
    assert!(edge_cases::NULL_BYTE.contains('\0'));
}
