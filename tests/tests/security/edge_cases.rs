use nyx_tests::security::edge_cases;

#[test]
fn edge_case_corpus_contains_unicode_and_control_chars() {
    let all = edge_cases::all_string_edge_cases();
    assert!(all.contains(&edge_cases::UNICODE_EMOJI));
    assert!(all.contains(&edge_cases::UNICODE_RTL));
    assert!(all.contains(&edge_cases::NULL_BYTE));
}

#[test]
fn edge_case_max_length_string_is_large_enough_for_stress() {
    let max = edge_cases::max_length_string();
    assert_eq!(max.len(), 100_000);
}

#[test]
fn edge_case_numeric_boundaries_are_defined() {
    let min_i32 = edge_cases::I32_MIN;
    let max_i32 = edge_cases::I32_MAX;
    let min_i64 = edge_cases::I64_MIN;
    let max_i64 = edge_cases::I64_MAX;

    assert!(min_i32.saturating_add(1) <= max_i32);
    assert!(min_i64.saturating_add(1) <= max_i64);
}
