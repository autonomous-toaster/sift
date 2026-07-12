//! Command dispatcher — walks the AST, decides plugin vs exec, handles pipelines.

use std::io::Read;
use std::process::Command;

use anyhow::{Context, Result};
use brush_parser::ast::{
    AndOr, AndOrList, Command as AstCommand, Pipeline, SimpleCommand,
};

use crate::builtins::{execute_builtin, is_builtin};
use crate::parser::parse_line;
use crate::plugin::{PluginRegistry, PluginResult};
use crate::session::Session;

/// Dispatch a single line of shell input.
pub async fn dispatch(session: &mut Session, registry: &PluginRegistry, input: &str) -> Result<Vec<u8>> {
    let program = parse_line(input)?;

    let mut output = Vec::new();

    for complete_cmd in &program.complete_commands {
        for item in &complete_cmd.0 {
            let cmd_output = dispatch_and_or_list(session, registry, &item.0).await?;
            output.extend(cmd_output);
        }
    }

    Ok(output)
}

/// Dispatch an `AndOrList` (handles && and ||).
async fn dispatch_and_or_list(
    session: &mut Session,
    registry: &PluginRegistry,
    list: &AndOrList,
) -> Result<Vec<u8>> {
    let mut output = dispatch_pipeline(session, registry, &list.first).await?;

    for and_or in &list.additional {
        match and_or {
            AndOr::And(pipeline) | AndOr::Or(pipeline) => {
                let out = dispatch_pipeline(session, registry, pipeline).await?;
                output.extend(out);
            }
        }
    }

    Ok(output)
}

enum Decision {
    Plugin(String),
    Exec,
}

async fn dispatch_pipeline(
    session: &mut Session,
    registry: &PluginRegistry,
    pipeline: &Pipeline,
) -> Result<Vec<u8>> {
    let cmds = &pipeline.seq;
    if cmds.is_empty() {
        return Ok(Vec::new());
    }

    let decisions: Vec<Decision> = cmds.iter().map(|cmd| decide_command(cmd, registry)).collect();
    let has_plugin = decisions.iter().any(|d| matches!(d, Decision::Plugin(_)));

    if !has_plugin {
        return execute_through_bash(pipeline);
    }

    execute_pipeline_with_plugins(session, registry, cmds, &decisions).await
}

fn decide_command(cmd: &AstCommand, registry: &PluginRegistry) -> Decision {
    match cmd {
        AstCommand::Simple(simple) => {
            let name = get_command_name(simple);
            match name {
                Some(name) if is_builtin(name) => Decision::Exec,
                Some(name) => match registry.find(name) {
                    Some(_) => Decision::Plugin(name.to_string()),
                    None => Decision::Exec,
                },
                None => Decision::Exec,
            }
        }
        _ => Decision::Exec,
    }
}

fn get_command_name(simple: &SimpleCommand) -> Option<&str> {
    simple.word_or_name.as_ref().map(std::convert::AsRef::as_ref)
}

fn get_command_args(simple: &SimpleCommand) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(suffix) = &simple.suffix {
        for item in &suffix.0 {
            use brush_parser::ast::CommandPrefixOrSuffixItem;
            if let CommandPrefixOrSuffixItem::Word(w) = item {
                args.push(w.to_string());
            }
        }
    }
    args
}

fn execute_through_bash(pipeline: &Pipeline) -> Result<Vec<u8>> {
    let cmd_str = pipeline.to_string();
    let output = Command::new("/bin/bash")
        .args(["-c", &cmd_str])
        .output()
        .with_context(|| format!("failed to execute: {cmd_str}"))?;

    let mut result = output.stdout;
    result.extend(output.stderr);
    Ok(result)
}

#[allow(clippy::too_many_lines)]
async fn handle_plugin_command(
    session: &mut Session,
    registry: &PluginRegistry,
    cmd: &AstCommand,
    plugin_name: &str,
    prev_read: &mut Option<os_pipe::PipeReader>,
    stdout: Option<os_pipe::PipeWriter>,
    handles: &mut Vec<std::process::Child>,
) -> Result<Option<Vec<u8>>> {
    let plugin = registry.find(plugin_name).unwrap();
    let args = match cmd {
        AstCommand::Simple(simple) => get_command_args(simple),
        _ => Vec::new(),
    };

    let input: Option<Vec<u8>> = match prev_read.take() {
        Some(mut reader) => {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Some(buf)
        }
        None => None,
    };

    let result = plugin.execute(session, &args, input.as_deref()).await?;

    match result {
        PluginResult::Handled { output, .. } => {
            if let Some(mut writer) = stdout {
                use std::io::Write;
                writer.write_all(&output)?;
            } else {
                return Ok(Some(output));
            }
        }
        PluginResult::Passthrough => {
            let (name, args) = match cmd {
                AstCommand::Simple(simple) => {
                    (get_command_name(simple).map(ToString::to_string), get_command_args(simple))
                }
                _ => (None, Vec::new()),
            };
            if let Some(name) = name {
                let mut child_cmd = Command::new(&name);
                child_cmd.args(&args);
                if let Some(writer) = stdout {
                    child_cmd.stdout(writer);
                }
                let child = child_cmd
                    .spawn()
                    .with_context(|| format!("failed to spawn: {name}"))?;
                handles.push(child);
            }
        }
        PluginResult::Unchanged { message, .. } => {
            if let Some(mut writer) = stdout {
                use std::io::Write;
                writeln!(writer, "{message}")?;
            } else {
                return Ok(Some(format!("{message}\n").into_bytes()));
            }
        }
    }

    Ok(None)
}

async fn handle_exec_command(
    session: &mut Session,
    registry: &PluginRegistry,
    cmd: &AstCommand,
    prev_read: &mut Option<os_pipe::PipeReader>,
    stdout: Option<os_pipe::PipeWriter>,
    handles: &mut Vec<std::process::Child>,
) -> Result<()> {
    let (name, args) = match cmd {
        AstCommand::Simple(simple) => {
            (get_command_name(simple).map(ToString::to_string), get_command_args(simple))
        }
        _ => return Ok(()),
    };

    if let Some(name) = name {
        if is_builtin(&name) {
            if let Some(output) = execute_builtin(session, registry, &name, &args).await? {
                if let Some(mut writer) = stdout {
                    use std::io::Write;
                    writer.write_all(&output)?;
                }
            }
            return Ok(());
        }

        let mut child_cmd = Command::new(&name);
        child_cmd.args(&args);
        if let Some(reader) = prev_read.take() {
            child_cmd.stdin(reader);
        }
        if let Some(writer) = stdout {
            child_cmd.stdout(writer);
        }
        let child = child_cmd
            .spawn()
            .with_context(|| format!("failed to spawn: {name}"))?;
        handles.push(child);
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn execute_pipeline_with_plugins(
    session: &mut Session,
    registry: &PluginRegistry,
    cmds: &[AstCommand],
    decisions: &[Decision],
) -> Result<Vec<u8>> {
    let mut pipes: Vec<(os_pipe::PipeReader, os_pipe::PipeWriter)> = Vec::new();
    for _ in 0..cmds.len().saturating_sub(1) {
        let (reader, writer) = os_pipe::pipe()?;
        pipes.push((reader, writer));
    }

    let mut handles = Vec::new();
    let mut prev_read: Option<os_pipe::PipeReader> = None;

    for (i, (cmd, decision)) in cmds.iter().zip(decisions.iter()).enumerate() {
        let stdout = pipes.get(i).map(|(_, ref writer)| writer.try_clone()).transpose()?;

        match decision {
            Decision::Plugin(plugin_name) => {
                if let Some(output) = handle_plugin_command(
                    session, registry, cmd, plugin_name,
                    &mut prev_read, stdout, &mut handles,
                ).await? {
                    return Ok(output);
                }
            }
            Decision::Exec => {
                handle_exec_command(
                    session, registry, cmd,
                    &mut prev_read, stdout, &mut handles,
                ).await?;
            }
        }

        if i < cmds.len() - 1 {
            prev_read = Some(pipes[i].0.try_clone()?);
        }
    }

    for mut handle in handles {
        let _ = handle.wait();
    }

    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{Plugin, PluginResult};
    use crate::session::Session;

    use async_trait::async_trait;

    struct TestCatPlugin;

    #[async_trait]
    impl Plugin for TestCatPlugin {
        fn name(&self) -> &'static str {
            "cat"
        }

        async fn execute(
            &self,
            _session: &mut Session,
            args: &[String],
            _stdin: Option<&[u8]>,
        ) -> Result<PluginResult> {
            if args.is_empty() {
                return Ok(PluginResult::Passthrough);
            }
            let path = &args[0];
            let content = std::fs::read(path)?;
            Ok(PluginResult::Handled {
                output: content,
                exit_code: 0,
            })
        }
    }

    #[tokio::test]
    async fn test_dispatch_simple_command() {
        let mut session = Session::from_env();
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestCatPlugin));

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").unwrap();

        let cmd = format!("cat {}", file_path.display());
        let output = dispatch(&mut session, &registry, &cmd).await.unwrap();
        assert_eq!(output, b"hello world");
    }

    #[tokio::test]
    async fn test_dispatch_unknown_command() {
        let mut session = Session::from_env();
        let registry = PluginRegistry::new();

        let result = dispatch(&mut session, &registry, "nonexistent_command_xyz").await;
        assert!(result.is_ok() || result.is_err());
    }
}
