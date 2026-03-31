//! New-app scaffold tests — filesystem only, no network.

use tempfile::TempDir;

fn scaffold(tmp: &TempDir, name: &str) -> anyhow::Result<()> {
    nyx_xtask::commands::new_app::run(name, tmp.path())
}

#[test]
fn creates_app_profiles_directory() {
    let tmp = tempfile::tempdir().unwrap();
    scaffold(&tmp, "Anteros").unwrap();
    assert!(tmp.path().join("apps/Anteros/Anteros-profiles/src").exists());
}

#[test]
fn creates_migrations_directory() {
    let tmp = tempfile::tempdir().unwrap();
    scaffold(&tmp, "Anteros").unwrap();
    assert!(tmp.path().join("migrations/Anteros").exists());
}

#[test]
fn creates_cargo_toml_stub() {
    let tmp = tempfile::tempdir().unwrap();
    scaffold(&tmp, "Anteros").unwrap();
    assert!(tmp
        .path()
        .join("apps/Anteros/Anteros-profiles/Cargo.toml")
        .exists());
}

#[test]
fn creates_main_rs_stub() {
    let tmp = tempfile::tempdir().unwrap();
    scaffold(&tmp, "Anteros").unwrap();
    assert!(tmp
        .path()
        .join("apps/Anteros/Anteros-profiles/src/lib.rs")
        .exists());
}

#[test]
fn rejects_invalid_app_name_with_spaces() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(scaffold(&tmp, "my app").is_err());
}

#[test]
fn rejects_empty_app_name() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(scaffold(&tmp, "").is_err());
}

#[test]
fn gitkeep_created_in_migrations_dir() {
    let tmp = tempfile::tempdir().unwrap();
    scaffold(&tmp, "Anteros").unwrap();
    assert!(tmp.path().join("migrations/Anteros/.gitkeep").exists());
}

#[test]
fn lib_rs_contains_service_comment() {
    let tmp = tempfile::tempdir().unwrap();
    scaffold(&tmp, "Anteros").unwrap();
    let content = std::fs::read_to_string(
        tmp.path()
            .join("apps/Anteros/Anteros-profiles/src/lib.rs"),
    )
    .unwrap();
    assert!(content.contains("Anteros"));
}

#[test]
fn cargo_toml_contains_package_name() {
    let tmp = tempfile::tempdir().unwrap();
    scaffold(&tmp, "Anteros").unwrap();
    let content = std::fs::read_to_string(
        tmp.path()
            .join("apps/Anteros/Anteros-profiles/Cargo.toml"),
    )
    .unwrap();
    assert!(content.contains("anteros-profiles"));
}

#[test]
fn rejects_name_with_special_chars() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(scaffold(&tmp, "my@app").is_err());
}
