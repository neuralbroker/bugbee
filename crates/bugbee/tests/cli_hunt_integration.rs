use std::io::Write;
use std::path::Path;
use std::process::{Command, Output};
use std::str;

fn bugbee_bin() -> std::path::PathBuf {
    std::env::var_os("CARGO_BIN_EXE_bugbee")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            // Fallback: assume we're in target/debug or release
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("bugbee")))
        })
        .expect("bugbee binary not found; set CARGO_BIN_EXE_bugbee or run via `cargo test`")
}

fn create_vulnerable_project(dir: &Path) {
    let src = dir.join("src");
    std::fs::create_dir_all(&src).unwrap();

    let main_rs = src.join("main.rs");
    let mut f = std::fs::File::create(&main_rs).unwrap();
    f.write_all(
        b"fn main() {\n\
          let user_input = \"admin\";\n\
          let query = format!(\"SELECT * FROM users WHERE name = '{input}'\", input = user_input);\n\
          println!(\"{}\", query);\n\
          }\n",
    )
    .unwrap();

    let cargo = dir.join("Cargo.toml");
    let mut f = std::fs::File::create(&cargo).unwrap();
    f.write_all(
        b"[package]\n\
          name = \"vulnerable-test\"\n\
          version = \"0.1.0\"\n\
          edition = \"2021\"\n\
          \n\
          [dependencies]\n",
    )
    .unwrap();
}

fn bugbee(args: &[&str], cwd: &Path) -> Output {
    Command::new(bugbee_bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

#[test]
fn test_hunt_integration() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    create_vulnerable_project(root);

    let out = bugbee(&["init"], root);
    let stdout = str::from_utf8(&out.stdout).unwrap();
    assert!(
        out.status.success(),
        "init failed:\nstdout: {}\nstderr: {}",
        stdout,
        str::from_utf8(&out.stderr).unwrap()
    );
    assert!(stdout.contains("initialized"));

    let out = bugbee(&["hunt"], root);
    let stdout = str::from_utf8(&out.stdout).unwrap();
    let stderr = str::from_utf8(&out.stderr).unwrap();
    assert!(
        out.status.success(),
        "hunt failed:\nstdout: {}\nstderr: {}",
        stdout,
        stderr
    );
    assert!(stdout.contains("hunt complete"));
    assert!(
        stdout.contains("findings"),
        "expected findings in output, got:\n{}",
        stdout
    );
}

#[test]
fn test_hunt_with_scope_file_integration() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    create_vulnerable_project(root);

    let out = bugbee(&["init"], root);
    assert!(out.status.success(), "init failed");

    let bugbee_dir = root.join(".bugbee");
    std::fs::create_dir_all(&bugbee_dir).unwrap();
    let scope_file = bugbee_dir.join("scope.toml");
    let mut f = std::fs::File::create(&scope_file).unwrap();
    f.write_all(
        b"[scope]\n\
          allowed_hosts = [\"*.example.com\", \"localhost\"]\n\
          allowed_url_prefixes = [\"http://localhost:8080\"]\n",
    )
    .unwrap();

    let out = bugbee(&["hunt"], root);
    assert!(
        out.status.success(),
        "hunt with scope file failed:\n{}",
        str::from_utf8(&out.stderr).unwrap()
    );
}

#[test]
fn test_init_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    create_vulnerable_project(root);

    let out1 = bugbee(&["init"], root);
    assert!(out1.status.success());
    let out2 = bugbee(&["init"], root);
    assert!(
        out2.status.success(),
        "second init failed:\n{}",
        str::from_utf8(&out2.stderr).unwrap()
    );
}

#[test]
fn test_doctor_command() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    create_vulnerable_project(root);

    let out = bugbee(&["init"], root);
    assert!(out.status.success());
    let out = bugbee(&["doctor"], root);
    let stdout = str::from_utf8(&out.stdout).unwrap();
    assert!(out.status.success(), "doctor failed:\n{}", stdout);
    assert!(stdout.contains("bugbee"));
    assert!(stdout.contains("tools:"));
}

#[test]
fn test_findings_list_after_hunt() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    create_vulnerable_project(root);

    bugbee(&["init"], root);
    let out = bugbee(&["hunt"], root);
    assert!(out.status.success(), "hunt failed");

    let out = bugbee(&["findings"], root);
    let stdout = str::from_utf8(&out.stdout).unwrap();
    assert!(
        out.status.success(),
        "findings failed:\n{}",
        stdout
    );
    assert!(
        stdout.contains("findings") || stdout.contains("total") || stdout.contains("no findings")
    );
}

#[test]
fn test_report_sarif() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    create_vulnerable_project(root);

    bugbee(&["init"], root);
    bugbee(&["hunt"], root);

    let out = bugbee(&["report", "--output", "test.sarif.json"], root);
    assert!(
        out.status.success(),
        "report failed:\n{}",
        str::from_utf8(&out.stderr).unwrap()
    );
    let report_path = root.join("test.sarif.json");
    assert!(report_path.is_file(), "SARIF report not written");
    let content = std::fs::read_to_string(&report_path).unwrap();
    assert!(
        content.contains("sarif") || content.contains("version"),
        "not valid SARIF"
    );
}
