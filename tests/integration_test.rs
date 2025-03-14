use assert_cmd::{Command, cargo::CommandCargoExt};
use std::process::{Child, Command as StdCommand};

#[test]
fn test_sample_db_info() {
    let mut cmd = Command::cargo_bin("RQLite").unwrap();
    cmd.arg("sample.db")
        .arg("db-info")
        .assert()
        .success()
        .stdout(predicates::str::contains("database page size: 4096"))
        .stdout(predicates::str::contains("database page count: 4"));
}

#[test]
fn test_sample_table() {
    let mut cmd = Command::cargo_bin("RQLite").unwrap();
    cmd.arg("sample.db")
        .arg("tables")
        .assert()
        .success()
        .stdout(predicates::str::contains("apples"))
        .stdout(predicates::str::contains("sqlite_sequence"))
        .stdout(predicates::str::contains("oranges"));
}

#[test]
fn test_web() {
    let mut cmd = StdCommand::cargo_bin("RQLite")
        .unwrap()
        .arg("sample.db")
        .arg("web")
        .spawn()
        .unwrap();
    cmd.kill().unwrap();
}
