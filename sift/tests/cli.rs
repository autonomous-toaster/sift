use assert_cmd::Command;

#[test]
fn test_agent_mode_echo() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo hello");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("hello"));
}

#[test]
fn test_agent_mode_exit_code() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("exit 42");
    cmd.assert().code(42);
}

#[test]
fn test_gain_flag() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("--gain");
    cmd.assert().success();
}

#[test]
fn test_shell_mode() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("--shell").write_stdin("exit\n").assert().success();
}
