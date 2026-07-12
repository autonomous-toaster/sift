//! Shell input parser — thin wrapper over brush-parser.

use anyhow::{Context, Result};
use brush_parser::ast::Program;
use brush_parser::{parse_tokens, tokenize_str_with_options, ParserOptions};

/// Parse a single line of shell input into a brush-parser `Program` AST.
///
/// Returns an error for unparseable input (unmatched quotes, invalid syntax, etc.).
pub fn parse_line(input: &str) -> Result<Program> {
    let options = ParserOptions::default();
    let tokens = tokenize_str_with_options(input, &options.tokenizer_options())
        .map_err(|e| anyhow::anyhow!("tokenization error: {e}"))?;
    parse_tokens(&tokens, &options).context("failed to parse shell command")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let program = parse_line("cat foo.rs").unwrap();
        assert!(!program.complete_commands.is_empty());
    }

    #[test]
    fn test_parse_pipeline() {
        let program = parse_line("cat foo | grep bar").unwrap();
        assert!(!program.complete_commands.is_empty());
    }

    #[test]
    fn test_parse_redirect() {
        let program = parse_line("cat foo > out.txt").unwrap();
        assert!(!program.complete_commands.is_empty());
    }

    #[test]
    fn test_parse_and_or() {
        let program = parse_line("make && make test").unwrap();
        assert!(!program.complete_commands.is_empty());
    }

    #[test]
    fn test_parse_env_var() {
        let program = parse_line("echo $HOME").unwrap();
        assert!(!program.complete_commands.is_empty());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let result = parse_line("echo 'unclosed");
        assert!(result.is_err());
    }
}
