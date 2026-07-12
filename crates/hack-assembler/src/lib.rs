pub mod codegen;
pub mod parser;
pub mod token;

use thiserror::Error;

pub use codegen::{CodegenError, generate};
pub use parser::{AValue, Comp, Dest, Instruction, Jump, ParseError, Parser};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AssembleError {
    #[error("unable to parse assembly: {0}")]
    Parse(#[from] ParseError),
    #[error("unable to generate machine code: {0}")]
    Codegen(#[from] CodegenError),
}

pub fn parse(source: &str) -> Result<Vec<Instruction>, ParseError> {
    Parser::new(source).parse_all()
}

pub fn assemble(source: &str) -> Result<String, AssembleError> {
    let instructions = parse(source)?;
    let output = generate(&instructions)?;
    Ok(output.join("\n") + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_source_with_a_trailing_newline() {
        assert_eq!(
            assemble("@2\nD=A\n"),
            Ok("0000000000000010\n1110110000010000\n".to_string())
        );
    }

    #[test]
    fn reports_the_failing_stage() {
        assert!(matches!(assemble("@32768"), Err(AssembleError::Parse(_))));
    }
}
