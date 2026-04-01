use arbitrary::{Arbitrary, Unstructured};
use assert_cmd::Command;
use mockall::mock;
use predicates::str::contains;
use rstest::rstest;
use serial_test::serial;
use tempfile::NamedTempFile;
use test_case::test_case;
use thiserror::Error;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Error)]
#[error("invalid token: {0}")]
struct TokenError(String);

trait Clock {
    fn now(&self) -> u64;
}

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    token: String,
    issuer: u32,
}

mock! {
    Clock {}
    impl Clock for Clock {
        fn now(&self) -> u64;
    }
}

#[test]
fn assert_cmd_and_predicates_validate_tooling() {
    let mut cmd = Command::new("rustc");
    cmd.arg("--version");
    cmd.assert().success().stdout(contains("rustc"));
}

#[rstest]
#[case("Bearer abc", true)]
#[case("", false)]
#[case("no-prefix", false)]
fn rstest_covers_token_shape(#[case] token: &str, #[case] valid: bool) {
    let observed = token.starts_with("Bearer ") && token.len() > 7;
    assert_eq!(observed, valid);
}

#[test_case("owner", "owner", true; "same owner")]
#[test_case("owner", "viewer", false; "different owner")]
fn test_case_covers_simple_authz(owner: &str, actor: &str, allowed: bool) {
    assert_eq!(owner == actor, allowed);
}

#[test]
#[serial]
fn serial_test_guards_tempfile_sequence() {
    let file = NamedTempFile::new().expect("tempfile should create");
    std::fs::write(file.path(), "serial-check").expect("write should succeed");
    let content = std::fs::read_to_string(file.path()).expect("read should succeed");
    assert_eq!(content, "serial-check");
}

#[test]
fn mockall_covers_trait_contract() {
    let mut clock = MockClock::new();
    clock.expect_now().times(1).return_const(42_u64);
    assert_eq!(clock.now(), 42);
}

#[test]
fn arbitrary_generates_struct_from_bytes() {
    let bytes = b"\x05hello\x00\x00\x00\x2a";
    let mut unstructured = Unstructured::new(bytes);
    let value = FuzzInput::arbitrary(&mut unstructured);
    assert!(value.is_ok());
    let generated = value.expect("arbitrary value should parse");
    let issuer = generated.issuer;
    let token_len = generated.token.len();
    assert!(issuer.saturating_add(1) >= issuer);
    assert!(token_len <= 1024);
}

#[test]
fn loom_models_simple_concurrency() {
    loom::model(|| {
        use loom::sync::atomic::{AtomicUsize, Ordering};
        use loom::sync::Arc;
        use loom::thread;

        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = Arc::clone(&counter);
        let c2 = Arc::clone(&counter);

        let t1 = thread::spawn(move || {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        let t2 = thread::spawn(move || {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        t1.join().expect("t1 should join");
        t2.join().expect("t2 should join");
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    });
}

quickcheck::quickcheck! {
    fn quickcheck_reverse_twice_is_identity(input: String) -> bool {
        let reversed: String = input.chars().rev().collect();
        let round_trip: String = reversed.chars().rev().collect();
        round_trip == input
    }
}

#[test]
fn insta_snapshot_captures_structured_output() {
    let payload = serde_json::json!({
        "suite": "tooling_coverage",
        "status": "ok",
        "checks": ["assert_cmd", "loom", "mockall", "quickcheck"]
    });
    insta::assert_yaml_snapshot!(payload, @r###"
    ---
    checks:
      - assert_cmd
      - loom
      - mockall
      - quickcheck
    status: ok
    suite: tooling_coverage
    "###);
}

#[test]
fn thiserror_and_pretty_assertions_are_wired() {
    let err = TokenError("bad".to_string());
    pretty_assertions::assert_eq!(err.to_string(), "invalid token: bad");
}

#[test]
fn tracing_stack_is_usable_in_tests() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .try_init();
    info!(target: "tests", "tracing coverage check");
}
