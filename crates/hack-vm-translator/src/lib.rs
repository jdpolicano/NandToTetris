//! Translator for the stack arithmetic and memory-access commands from
//! nand2tetris Project 7.
//!
//! [`translate`] is the convenient one-shot entry point. For callers that parse
//! or construct commands incrementally, create a [`Codegen`] for one VM file,
//! call [`Codegen::emit`] in source order, and consume it with
//! [`Codegen::finish`]. The file stem supplied to the generator defines the
//! static-symbol namespace and is validated before any instructions are
//! emitted.
//!
//! Project 8 program flow, functions, calls, bootstrap code, directory
//! translation, and multi-file orchestration are outside this crate's current
//! supported scope.

mod codegen;
mod parser;
mod token;
mod vm;

use hack_assembler::Instruction;
use thiserror::Error;

pub use codegen::{Codegen, CodegenError};
pub use hack_source::{SourcePosition, SourceSpan};
pub use parser::{ParseError, ParseErrorKind, VmCommand};
pub use vm::{Arithmetic, Segment};

/// An error produced while parsing or generating a VM translation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TranslateError {
    #[error("unable to parse VM source: {0}")]
    Parse(#[from] ParseError),
    #[error("unable to generate assembly: {0}")]
    Codegen(#[from] CodegenError),
}

/// Translates one Project 7 VM source file into typed Hack instructions.
///
/// `file_stem` identifies the file-local static-symbol namespace and must be a
/// valid Hack symbol that does not use the translator's reserved namespace.
pub fn translate(
    source: &str,
    file_stem: impl Into<String>,
) -> Result<Vec<Instruction>, TranslateError> {
    let commands = parser::Parser::new(source).parse_all()?;
    let mut codegen = codegen::Codegen::new(file_stem)?;
    for command in &commands {
        codegen.emit(command)?;
    }
    Ok(codegen.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn parser_never_panics_for_arbitrary_bounded_text(
            source in prop::collection::vec(any::<char>(), 0..=2048)
                .prop_map(|characters| characters.into_iter().collect::<String>())
        ) {
            let _ = parser::Parser::new(&source).parse_all();
        }
    }

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
