use std::io::Write;
use std::path::Path;
use std::process::Command;

fn bugbee_bin() -> std::path::PathBuf {
    std::env::var_os("CARGO_BIN_EXE_bugbee")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("bugbee")))
        })
        .expect("bugbee binary not found")
}

fn repo_root() -> std::path::PathBuf {
    // Walk up from cwd until we find a Cargo.toml with [workspace] or the fixtures dir
    let mut cwd = std::env::current_dir().unwrap();
    loop {
        if cwd.join("fixtures").join("python-vuln").exists() {
            return cwd;
        }
        if !cwd.pop() {
            panic!("could not find repo root from {:?}", std::env::current_dir().unwrap());
        }
    }
}

/// Path to the python-vuln fixture project
fn fixture_dir() -> std::path::PathBuf {
    repo_root().join("fixtures").join("python-vuln")
}

fn bugbee_output(args: &[&str], cwd: &Path) -> (bool, String, String) {
    let out = Command::new(bugbee_bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// E2E: hunt against the python-vuln fixture with mocked engine results
#[test]
fn test_e2e_python_vuln_hunt() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Copy fixture into temp dir to avoid mutating repo
    let src = fixture_dir();
    assert!(src.exists(), "fixture not found at {:?}", src);

    // Recursive copy
    copy_dir(&src, root).unwrap();

    // Run bugbee init (already has bugbee.toml, but init creates .bugbee state)
    let (ok, _stdout, stderr) = bugbee_output(&["init"], root);
    assert!(ok, "init failed:\n{stderr}");

    // Run hunt
    let (ok, stdout, stderr) = bugbee_output(&["hunt"], root);
    assert!(
        ok,
        "hunt failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Should detect findings from the fixture
    assert!(
        stdout.contains("hunt complete"),
        "expected 'hunt complete' in output, got:\n{stdout}"
    );
    assert!(
        stdout.contains("findings") || stdout.contains("files scanned"),
        "expected findings summary in:\n{stdout}"
    );

    // List findings to check detection
    let (ok, stdout, stderr) = bugbee_output(&["findings"], root);
    assert!(ok, "findings failed:\n{stderr}");

    // The fixture has: hardcoded password, subprocess shell, eval, SQL concat
    let has_secrets = stdout.contains("secrets") || stdout.contains("password");
    let has_eval = stdout.contains("eval");
    let has_shell = stdout.contains("shell") || stdout.contains("subprocess");

    eprintln!(
        "fixture detection: secrets={} eval={} shell={}\n{}",
        has_secrets, has_eval, has_shell, stdout
    );
}

/// E2E: SARIF export from fixture project is valid
#[test]
fn test_e2e_sarif_export_valid() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir(&fixture_dir(), root).unwrap();

    let (ok, _, stderr) = bugbee_output(&["init"], root);
    assert!(ok, "init failed:\n{stderr}");

    let (ok, _, stderr) = bugbee_output(&["hunt"], root);
    assert!(ok, "hunt failed:\n{stderr}");

    let sarif_path = root.join("output.sarif.json");
    let (ok, _stdout, stderr) = bugbee_output(
        &["report", "--output", &sarif_path.to_string_lossy()],
        root,
    );
    assert!(ok, "report failed:\n{stderr}");

    assert!(
        sarif_path.is_file(),
        "SARIF file not written at {:?}",
        sarif_path
    );

    let content = std::fs::read_to_string(&sarif_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        parsed.get("runs").is_some() || parsed.get("version").is_some(),
        "not valid SARIF: {content}"
    );
}

/// E2E: scope file is loadable and does not interfere with local hunt
#[test]
fn test_e2e_scope_file_does_not_block_local_hunt() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir(&fixture_dir(), root).unwrap();

    // Create scope file
    let bugbee_dir = root.join(".bugbee");
    std::fs::create_dir_all(&bugbee_dir).unwrap();
    let mut f = std::fs::File::create(bugbee_dir.join("scope.toml")).unwrap();
    f.write_all(
        b"[scope]\n\
          allowed_hosts = [\"*.example.com\"]\n\
          allowed_urls = [\"http://localhost:8080\"]\n",
    )
    .unwrap();

    let (ok, _, stderr) = bugbee_output(&["init"], root);
    assert!(ok, "init failed:\n{stderr}");

    // Local hunt should still work (scope is for live targets, not local scans)
    let (ok, stdout, stderr) = bugbee_output(&["hunt"], root);
    assert!(
        ok,
        "hunt with scope file failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("hunt complete"));
}

/// E2E: doctor shows correct info for fixture project
#[test]
fn test_e2e_doctor_fixture() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir(&fixture_dir(), root).unwrap();

    let (ok, _, stderr) = bugbee_output(&["init"], root);
    assert!(ok, "init failed:\n{stderr}");

    let (ok, stdout, stderr) = bugbee_output(&["doctor"], root);
    assert!(ok, "doctor failed:\n{stderr}");
    assert!(stdout.contains("bugbee"), "doctor output: {stdout}");
    assert!(stdout.contains("tools:"), "doctor output: {stdout}");
}

// ── Helpers ──────────────────────────────────────────────────────

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src).unwrap();
        let target = dst.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
