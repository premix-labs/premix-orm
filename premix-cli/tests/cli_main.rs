use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn bin_path() -> PathBuf {
    let exe = std::env::current_exe().expect("failed to locate test binary");
    let target_dir = exe
        .parent()
        .and_then(|p| p.parent())
        .expect("unexpected test binary path");
    let candidate = target_dir.join("premix-cli.exe");
    if candidate.exists() {
        candidate
    } else {
        target_dir.join("premix-cli")
    }
}

fn make_temp_dir() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("premix_cli_bin_test_{}", nanos));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cli_main_runs_init() {
    let status = Command::new(bin_path())
        .arg("init")
        .status()
        .expect("failed to run premix-cli");
    assert!(status.success());
}

#[test]
fn cli_main_runs_migrate_up() {
    let root = make_temp_dir();
    let migrations = root.join("migrations");
    fs::create_dir_all(&migrations).unwrap();
    let file = migrations.join("20260109000000_create_items.sql");
    fs::write(
        &file,
        "-- up\nCREATE TABLE items (id INTEGER PRIMARY KEY);\n-- down\nDROP TABLE items;\n",
    )
    .unwrap();

    let db_path = root.join("test.db");
    let db_url = format!("sqlite:{}", db_path.to_string_lossy().replace('\\', "/"));

    let status = Command::new(bin_path())
        .current_dir(&root)
        .arg("migrate")
        .arg("up")
        .arg("--database")
        .arg(db_url)
        .status()
        .expect("failed to run premix-cli");
    assert!(status.success());

    let _ = fs::remove_dir_all(&root);
}
