pub mod codegen;
pub mod parser;
pub mod token;
pub mod vm;

use hack_assembler::Instruction;
use thiserror::Error;

use crate::{codegen::CodegenError, parser::ParseError};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TranslateError {
    #[error("unable to parse VM source: {0}")]
    Parse(#[from] ParseError),
    #[error("unable to generate assembly: {0}")]
    Codegen(#[from] CodegenError),
}

pub fn translate(
    source: &str,
    file_name: impl Into<String>,
) -> Result<Vec<Instruction>, TranslateError> {
    let commands = parser::Parser::new(source).parse_all()?;
    Ok(codegen::Codegen::new(file_name).generate(commands)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_source_into_structured_instructions() {
        let instructions = translate("push static 3\nneg\n", "Foo").unwrap();
        assert_eq!(
            hack_assembler::format_instructions(&instructions),
            "// push static 3\n@Foo.3\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n// neg\n@SP\nA=M-1\nM=-M\n"
        );
    }

    #[test]
    fn reports_the_translation_stage_that_failed() {
        assert!(matches!(
            translate("push", "Foo"),
            Err(TranslateError::Parse(_))
        ));
        assert!(matches!(
            translate("pop pointer 2", "Foo"),
            Err(TranslateError::Codegen(CodegenError::InvalidPointerIndex(
                2
            )))
        ));
    }
}
