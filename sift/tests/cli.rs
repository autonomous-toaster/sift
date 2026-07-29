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
fn test_gain_subcommand() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("gain");
    cmd.assert().success();
}

#[test]
fn test_gain_subcommand_with_flags() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("gain").arg("--daily");
    cmd.assert().success();
}

#[test]
fn test_shell_mode() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("--shell").write_stdin("exit\n").assert().success();
}

#[test]
fn test_default_repl() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.write_stdin("exit\n").assert().success();
}

#[test]
fn test_agent_mode_failed_pipeline() {
    // When the preceding command in a pipeline fails, stderr+stdout should be visible
    // The shell plugin emits a nudge with the path to the saved output
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo hello | grep nonexistent");
    cmd.assert()
        .code(1)
        .stdout(predicates::str::contains("[nudge] error output saved. raw:"));
}

#[test]
fn test_agent_mode_pipeline_with_variable() {
    // Variable expansion should work in piped commands
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo \"$HOME\" | head");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/"));
}

#[test]
fn test_agent_mode_epipe_resilience() {
    // head should not cause EPIPE crash when reading large input
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("seq 1 10000 | head -5");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("1\n2\n3\n4\n5"));
}

#[test]
fn test_scred_plugin_echo() {
    // The shell plugin handles echo commands (falls through to __default__).
    // When scred feature is enabled, output is redacted via sift.ext.scred.
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo hello from scred test");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("hello from scred test"));
}

#[test]
fn test_scred_plugin_env() {
    // The shell plugin handles env commands.
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("env");
    cmd.assert().success();
}

#[test]
fn test_shell_plugin_default_fallback() {
    // The shell plugin (pattern = __default__) handles commands that don't
    // match any other plugin. It should execute them normally.
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("whoami");
    cmd.assert().success();
}

#[test]
fn test_shell_metachar_semicolon() {
    // Commands with ; should split on the metacharacter and expand $? correctly
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo hello; echo \"exit: $?\"");
    cmd.assert().success().stdout(predicates::str::contains("hello"));
    cmd.assert().success().stdout(predicates::str::contains("exit: 0"));
}

#[test]
fn test_shell_metachar_and() {
    // Commands with && should split on the metacharacter
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo first && echo second");
    cmd.assert().success().stdout(predicates::str::contains("first"));
    cmd.assert().success().stdout(predicates::str::contains("second"));
}

#[test]
fn test_shell_metachar_semicolon_inside_quotes() {
    // ; inside quotes should not cause a split
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo \"hello; world\"");
    cmd.assert().success().stdout(predicates::str::contains("hello; world"));
}

#[test]
fn test_shell_metachar_sed_then_echo() {
    // sed plugin should match the first segment before ;
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo hello; echo done");
    cmd.assert().success().stdout(predicates::str::contains("hello"));
    cmd.assert().success().stdout(predicates::str::contains("done"));
}
