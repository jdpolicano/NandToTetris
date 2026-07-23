//! Parser, formatter, and machine-code generator for the Hack assembly language.
//!
//! This crate implements the assembler portion of nand2tetris Project 6. Use
//! [`parse`] when typed [`Instruction`] values are needed, [`assemble`] for the
//! complete source-to-machine-code path, and [`assemble_instructions`] when a
//! preceding compiler stage already produces typed instructions.
//!
//! [`Instruction`] intentionally includes labels and comments alongside
//! executable A- and C-instructions. Keeping one ordered representation makes
//! parsing, formatting, and VM translation straightforward; these variants can
//! be separated if a future semantic analysis or optimization pass needs an
//! executable-only representation.

mod codegen;
mod instruction;
mod parser;
mod token;

use thiserror::Error;

pub use codegen::{CodegenError, generate};
pub use hack_source::{SourcePosition, SourceSpan};
pub use instruction::{AValue, Comp, Dest, Instruction, InstructionError, Jump, PredefinedSymbol};
pub use parser::{ParseError, ParseErrorKind};

/// An error produced while parsing or generating a Hack program.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AssembleError {
    #[error("unable to parse assembly {0}")]
    Parse(#[from] ParseError),
    #[error("unable to generate machine code {0}")]
    Codegen(#[from] CodegenError),
}

/// Parses Hack assembly source into its ordered, typed representation.
pub fn parse(source: &str) -> Result<Vec<Instruction>, ParseError> {
    parser::Parser::new(source).parse_all()
}

/// Assembles Hack assembly source into newline-terminated 16-bit binary words.
pub fn assemble(source: &str) -> Result<String, AssembleError> {
    let instructions = parse(source)?;
    Ok(assemble_instructions(&instructions)?)
}

/// Generates machine code from typed Hack instructions.
///
/// Labels and comments affect formatting and symbol resolution as appropriate,
/// but do not produce machine-code words.
pub fn assemble_instructions(instructions: &[Instruction]) -> Result<String, CodegenError> {
    let output = generate(instructions)?;
    Ok(if output.is_empty() {
        String::new()
    } else {
        output.join("\n") + "\n"
    })
}

/// Formats typed instructions as canonical Hack assembly, one item per line.
pub fn format_instructions(instructions: &[Instruction]) -> String {
    use std::fmt::Write;

    let mut output = String::new();
    for instruction in instructions {
        writeln!(&mut output, "{instruction}").expect("writing to a String cannot fail");
    }
    output
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
            let _ = parse(&source);
        }

        #[test]
        fn successful_parses_round_trip_through_canonical_format(
            source in prop::collection::vec(any::<char>(), 0..=2048)
                .prop_map(|characters| characters.into_iter().collect::<String>())
        ) {
            if let Ok(parsed) = parse(&source) {
                let formatted = format_instructions(&parsed);
                prop_assert_eq!(parse(&formatted), Ok(parsed));
            }
        }
    }

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

    #[test]
    fn displays_canonical_instructions() {
        assert_eq!(Instruction::A(AValue::Number(7)).to_string(), "@7");
        assert_eq!(
            Instruction::A(AValue::Symbol("SP".to_string())).to_string(),
            "@SP"
        );
        assert_eq!(
            Instruction::C {
                dest: Some(Dest::D),
                comp: Comp::MPlusOne,
                jump: None,
            }
            .to_string(),
            "D=M+1"
        );
        assert_eq!(
            Instruction::C {
                dest: None,
                comp: Comp::Zero,
                jump: Some(Jump::Jmp),
            }
            .to_string(),
            "0;JMP"
        );
        assert_eq!(
            Instruction::C {
                dest: Some(Dest::AMD),
                comp: Comp::DOrM,
                jump: Some(Jump::Jne),
            }
            .to_string(),
            "AMD=D|M;JNE"
        );
        assert_eq!(Instruction::Label("LOOP".to_string()).to_string(), "(LOOP)");
    }

    #[test]
    fn formats_instruction_sequences() {
        let instructions = vec![
            Instruction::A(AValue::Number(7)),
            Instruction::C {
                dest: Some(Dest::D),
                comp: Comp::A,
                jump: None,
            },
            Instruction::A(AValue::Symbol("SP".to_string())),
            Instruction::C {
                dest: Some(Dest::M),
                comp: Comp::D,
                jump: None,
            },
        ];

        assert_eq!(format_instructions(&instructions), "@7\nD=A\n@SP\nM=D\n");
        assert_eq!(format_instructions(&[]), "");
    }

    #[test]
    fn parsed_program_round_trips_through_display() {
        let parsed = parse("@7\nD=A\n(LOOP)\n@LOOP\n0;JMP\n").unwrap();
        let reparsed = parse(&format_instructions(&parsed)).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn structured_instructions_assemble_like_source() {
        let structured = vec![
            Instruction::A(AValue::Number(2)),
            Instruction::C {
                dest: Some(Dest::D),
                comp: Comp::A,
                jump: None,
            },
        ];

        assert_eq!(
            assemble("@2\nD=A\n").unwrap(),
            assemble_instructions(&structured).unwrap()
        );
    }

    #[test]
    fn validated_constructors_reject_invalid_values() {
        assert_eq!(
            Instruction::a_number(32_768),
            Err(InstructionError::AddressOutOfRange(32_768))
        );
        assert_eq!(
            Instruction::a_symbol("2bad"),
            Err(InstructionError::InvalidSymbol("2bad".to_string()))
        );
        assert_eq!(Instruction::a_symbol("SP").unwrap().to_string(), "@SP");
        assert_eq!(
            Instruction::a_symbol("SP").unwrap(),
            Instruction::a_predefined(PredefinedSymbol::SP)
        );
        assert_eq!(
            assemble_instructions(&[Instruction::a_symbol("SP").unwrap()]).unwrap(),
            "0000000000000000\n"
        );
    }

    #[test]
    fn predefined_symbols_are_infallible_and_map_to_hack_addresses() {
        let symbols = [
            (PredefinedSymbol::SP, "SP", 0),
            (PredefinedSymbol::LCL, "LCL", 1),
            (PredefinedSymbol::ARG, "ARG", 2),
            (PredefinedSymbol::THIS, "THIS", 3),
            (PredefinedSymbol::THAT, "THAT", 4),
            (PredefinedSymbol::R0, "R0", 0),
            (PredefinedSymbol::R1, "R1", 1),
            (PredefinedSymbol::R2, "R2", 2),
            (PredefinedSymbol::R3, "R3", 3),
            (PredefinedSymbol::R4, "R4", 4),
            (PredefinedSymbol::R5, "R5", 5),
            (PredefinedSymbol::R6, "R6", 6),
            (PredefinedSymbol::R7, "R7", 7),
            (PredefinedSymbol::R8, "R8", 8),
            (PredefinedSymbol::R9, "R9", 9),
            (PredefinedSymbol::R10, "R10", 10),
            (PredefinedSymbol::R11, "R11", 11),
            (PredefinedSymbol::R12, "R12", 12),
            (PredefinedSymbol::R13, "R13", 13),
            (PredefinedSymbol::R14, "R14", 14),
            (PredefinedSymbol::R15, "R15", 15),
            (PredefinedSymbol::SCREEN, "SCREEN", 16_384),
            (PredefinedSymbol::KEYBOARD, "KEYBOARD", 24_576),
        ];

        for (symbol, name, address) in symbols {
            assert_eq!(symbol.to_string(), name);
            assert_eq!(symbol.address(), address);
            assert_eq!(
                Instruction::a_predefined(symbol).to_string(),
                format!("@{name}")
            );
        }
    }

    #[test]
    fn direct_generic_predefined_symbol_still_uses_its_reserved_address() {
        let instruction = Instruction::A(AValue::Symbol("SP".to_string()));
        assert_eq!(
            assemble_instructions(&[instruction]).unwrap(),
            "0000000000000000\n"
        );
    }

    #[test]
    fn comments_format_as_assembly_and_do_not_generate_machine_code() {
        let instructions = [
            Instruction::Comment("push constant 7".to_string()),
            Instruction::A(AValue::Number(7)),
        ];

        assert_eq!(
            format_instructions(&instructions),
            "// push constant 7\n@7\n"
        );
        assert_eq!(
            assemble_instructions(&instructions).unwrap(),
            "0000000000000111\n"
        );
    }

    #[test]
    fn handles_comments_at_eof_and_crlf_without_panicking() {
        assert_eq!(
            parse("@1\r\nD=A\r\n// trailing comment"),
            parse("@1\nD=A\n")
        );
    }

    #[test]
    fn rejects_unicode_and_long_invalid_words_without_panicking() {
        assert!(parse("@λ\n").is_err());
        assert!(parse(&format!("@{}\n", "x".repeat(16_384))).is_ok());
        assert!(parse(&format!("{}\n", "unknown".repeat(4_096))).is_err());
    }
}
