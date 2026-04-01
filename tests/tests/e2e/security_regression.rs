use nyx_tests::security::{sql_injection, xss};

#[test]
fn security_regression_corpus_contains_expected_attack_vectors() {
    let sqli = sql_injection::all_payloads();
    let xss_payloads = xss::all_payloads();

    assert!(sqli.iter().any(|p| p.contains("DROP TABLE")));
    assert!(xss_payloads.iter().any(|p| p.contains("<script")));
}
