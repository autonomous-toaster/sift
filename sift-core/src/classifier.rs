//! Command classification — parse input and determine command structure.

use brush_parser::ast::{Command as AstCommand, Program, SimpleCommand};
use brush_parser::{parse_tokens, tokenize_str_with_options, ParserOptions};

/// Classification result with parsed information.
#[derive(Debug, Clone)]
pub struct Classification {
    /// The command name (e.g., "cat", "git").
    pub name: String,
    /// Command arguments.
    pub args: Vec<String>,
    /// Whether the command is part of a pipeline.
    pub is_piped: bool,
    /// Whether the command is part of a compound expression (&&, ||).
    pub is_compound: bool,
}

/// Parse and classify a shell command.
#[must_use]
pub fn classify(input: &str) -> Classification {
    let Some(program) = parse_input(input) else {
        return Classification {
            name: String::new(),
            args: Vec::new(),
            is_piped: false,
            is_compound: false,
        };
    };

    // Check if compound (&&, ||) — more than one AndOr in any list
    let is_compound = program
        .complete_commands
        .iter()
        .any(|cc| cc.0.iter().any(|item| !item.0.additional.is_empty()));

    // Get the first simple command
    let (name, args, is_piped) = extract_first_command(&program);

    Classification {
        name,
        args,
        is_piped,
        is_compound,
    }
}

/// Parse input using brush-parser's tokenize + parse pipeline.
fn parse_input(input: &str) -> Option<Program> {
    let options = ParserOptions::default();
    let tokens = tokenize_str_with_options(input, &options.tokenizer_options()).ok()?;
    parse_tokens(&tokens, &options).ok()
}

/// Extract the first command name, arguments, and pipeline status.
fn extract_first_command(program: &Program) -> (String, Vec<String>, bool) {
    for complete_cmd in &program.complete_commands {
        for item in &complete_cmd.0 {
            let and_or_list = &item.0;
            let pipeline = &and_or_list.first;
            let is_piped = pipeline.seq.len() > 1;

            if let Some(AstCommand::Simple(simple)) = pipeline.seq.first() {
                let name = get_command_name(simple).unwrap_or("").to_string();
                let args = get_command_args(simple);
                return (name, args, is_piped);
            }
        }
    }
    (String::new(), Vec::new(), false)
}

/// Get the command name from a `SimpleCommand`.
fn get_command_name(simple: &SimpleCommand) -> Option<&str> {
    simple
        .word_or_name
        .as_ref()
        .map(std::convert::AsRef::as_ref)
}

/// Get the arguments from a `SimpleCommand` (excluding the command name).
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

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_simple_cat() {
        let result = classify("cat foo.rs");
        assert_eq!(result.name, "cat");
        assert_eq!(result.args, vec!["foo.rs"]);
        assert!(!result.is_piped);
        assert!(!result.is_compound);
    }

    #[test]
    fn test_classify_cat_with_flags() {
        let result = classify("cat -n foo.rs");
        assert_eq!(result.name, "cat");
        assert_eq!(result.args, vec!["-n", "foo.rs"]);
    }

    #[test]
    fn test_classify_cat_piped() {
        let result = classify("cat foo.rs | grep bar");
        assert!(result.is_piped);
        assert_eq!(result.name, "cat");
    }

    #[test]
    fn test_classify_cargo_test() {
        let result = classify("cargo test");
        assert_eq!(result.name, "cargo");
        assert_eq!(result.args, vec!["test"]);
    }

    #[test]
    fn test_classify_git_status() {
        let result = classify("git status");
        assert_eq!(result.name, "git");
        assert_eq!(result.args, vec!["status"]);
    }

    #[test]
    fn test_classify_cd() {
        let result = classify("cd /tmp");
        assert_eq!(result.name, "cd");
        assert_eq!(result.args, vec!["/tmp"]);
    }

    #[test]
    fn test_classify_unknown() {
        let result = classify("gcc main.c");
        assert_eq!(result.name, "gcc");
        assert_eq!(result.args, vec!["main.c"]);
    }

    #[test]
    fn test_classify_compound() {
        let result = classify("cd /x && cat foo.rs");
        assert!(result.is_compound);
        assert_eq!(result.name, "cd");
    }

    #[test]
    fn test_classify_empty() {
        let result = classify("");
        assert!(result.name.is_empty());
        assert!(!result.is_piped);
        assert!(!result.is_compound);
    }
}
