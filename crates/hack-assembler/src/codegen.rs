use std::collections::HashMap;
use thiserror::Error;

use crate::instruction::{
    AValue, Comp, Dest, Instruction, Jump, PredefinedSymbol, predefined_symbol_address,
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CodegenError {
    #[error("duplicate label `{0}`")]
    DuplicateLabel(String),
    #[error("address space exhausted while resolving `{0}`")]
    AddressOverflow(String),
    #[error("variable address space exhausted while resolving `{0}`")]
    VariableAddressSpaceExhausted(String),
}

pub fn generate(instructions: &[Instruction]) -> Result<Vec<String>, CodegenError> {
    let labels = collect_labels(instructions)?;
    let mut variables = HashMap::new();
    let mut next_variable_address = 16;
    let mut output = Vec::new();

    for instruction in instructions {
        match instruction {
            Instruction::A(value) => {
                let address =
                    resolve_a_value(value, &labels, &mut variables, &mut next_variable_address)?;
                output.push(format!("{address:016b}"));
            }
            Instruction::C { dest, comp, jump } => {
                output.push(format!(
                    "111{}{}{}",
                    comp_bits(*comp),
                    dest_bits(*dest),
                    jump_bits(*jump)
                ));
            }
            Instruction::Label(_) | Instruction::Comment(_) => {}
        }
    }

    Ok(output)
}

fn collect_labels(instructions: &[Instruction]) -> Result<HashMap<String, u16>, CodegenError> {
    let mut labels = HashMap::new();
    let mut address = 0;

    for instruction in instructions {
        match instruction {
            Instruction::Label(symbol) => {
                if labels.insert(symbol.clone(), address).is_some() {
                    return Err(CodegenError::DuplicateLabel(symbol.clone()));
                }
            }
            Instruction::A(_) | Instruction::C { .. } => {
                if address == 32_768 {
                    return Err(CodegenError::AddressOverflow(address.to_string()));
                }
                address += 1;
            }
            Instruction::Comment(_) => {}
        }
    }

    Ok(labels)
}

fn resolve_a_value(
    value: &AValue,
    labels: &HashMap<String, u16>,
    variables: &mut HashMap<String, u16>,
    next_variable_address: &mut u16,
) -> Result<u16, CodegenError> {
    match value {
        AValue::Number(number) => Ok(*number),
        AValue::Predefined(symbol) => Ok(symbol.address()),
        AValue::Symbol(symbol) => {
            if let Some(address) = predefined_symbol_address(symbol) {
                return Ok(address);
            }

            if let Some(address) = labels.get(symbol) {
                return Ok(*address);
            }

            if let Some(address) = variables.get(symbol) {
                return Ok(*address);
            }

            if *next_variable_address >= PredefinedSymbol::SCREEN.address() {
                return Err(CodegenError::VariableAddressSpaceExhausted(symbol.clone()));
            }

            let address = *next_variable_address;
            variables.insert(symbol.clone(), address);
            *next_variable_address += 1;
            Ok(address)
        }
    }
}

fn dest_bits(dest: Option<Dest>) -> &'static str {
    match dest {
        None => "000",
        Some(Dest::M) => "001",
        Some(Dest::D) => "010",
        Some(Dest::MD) => "011",
        Some(Dest::A) => "100",
        Some(Dest::AM) => "101",
        Some(Dest::AD) => "110",
        Some(Dest::AMD) => "111",
    }
}

fn comp_bits(comp: Comp) -> &'static str {
    match comp {
        Comp::Zero => "0101010",
        Comp::One => "0111111",
        Comp::NegOne => "0111010",
        Comp::D => "0001100",
        Comp::A => "0110000",
        Comp::M => "1110000",
        Comp::NotD => "0001101",
        Comp::NotA => "0110001",
        Comp::NotM => "1110001",
        Comp::NegD => "0001111",
        Comp::NegA => "0110011",
        Comp::NegM => "1110011",
        Comp::DPlusOne => "0011111",
        Comp::APlusOne => "0110111",
        Comp::MPlusOne => "1110111",
        Comp::DMinusOne => "0001110",
        Comp::AMinusOne => "0110010",
        Comp::MMinusOne => "1110010",
        Comp::DPlusA => "0000010",
        Comp::DPlusM => "1000010",
        Comp::DMinusA => "0010011",
        Comp::DMinusM => "1010011",
        Comp::AMinusD => "0000111",
        Comp::MMinusD => "1000111",
        Comp::DAndA => "0000000",
        Comp::DAndM => "1000000",
        Comp::DOrA => "0010101",
        Comp::DOrM => "1010101",
    }
}

fn jump_bits(jump: Option<Jump>) -> &'static str {
    match jump {
        None => "000",
        Some(Jump::Jgt) => "001",
        Some(Jump::Jeq) => "010",
        Some(Jump::Jge) => "011",
        Some(Jump::Jlt) => "100",
        Some(Jump::Jne) => "101",
        Some(Jump::Jle) => "110",
        Some(Jump::Jmp) => "111",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn assemble(input: &str) -> Result<Vec<String>, CodegenError> {
        let instructions = Parser::new(input).parse_all().unwrap();
        generate(&instructions)
    }

    #[test]
    fn generates_a_and_c_instructions() {
        assert_eq!(
            assemble("@2\nD=A\n@3\nD=D+A\n@0\nM=D\n"),
            Ok(vec![
                "0000000000000010".to_string(),
                "1110110000010000".to_string(),
                "0000000000000011".to_string(),
                "1110000010010000".to_string(),
                "0000000000000000".to_string(),
                "1110001100001000".to_string(),
            ])
        );
    }

    #[test]
    fn resolves_labels() {
        assert_eq!(
            assemble("@LOOP\n0;JMP\n(LOOP)\n@LOOP\n0;JMP\n"),
            Ok(vec![
                "0000000000000010".to_string(),
                "1110101010000111".to_string(),
                "0000000000000010".to_string(),
                "1110101010000111".to_string(),
            ])
        );
    }

    #[test]
    fn allocates_variables_from_address_16() {
        assert_eq!(
            assemble("@i\nM=1\n@sum\nM=0\n@i\nD=M\n"),
            Ok(vec![
                "0000000000010000".to_string(),
                "1110111111001000".to_string(),
                "0000000000010001".to_string(),
                "1110101010001000".to_string(),
                "0000000000010000".to_string(),
                "1111110000010000".to_string(),
            ])
        );
    }

    #[test]
    fn rejects_duplicate_labels() {
        let instructions = Parser::new("(LOOP)\n@0\n(LOOP)\n@1\n").parse_all().unwrap();

        assert_eq!(
            generate(&instructions),
            Err(CodegenError::DuplicateLabel("LOOP".to_string()))
        );
    }

    #[test]
    fn rejects_variables_that_would_overlap_screen_memory() {
        let labels = HashMap::new();
        let mut variables = HashMap::new();
        let mut next_variable_address = PredefinedSymbol::SCREEN.address() - 1;

        assert_eq!(
            resolve_a_value(
                &AValue::Symbol("last_variable".to_string()),
                &labels,
                &mut variables,
                &mut next_variable_address,
            ),
            Ok(16_383)
        );
        assert_eq!(next_variable_address, PredefinedSymbol::SCREEN.address());

        assert_eq!(
            resolve_a_value(
                &AValue::Symbol("overflow".to_string()),
                &labels,
                &mut variables,
                &mut next_variable_address,
            ),
            Err(CodegenError::VariableAddressSpaceExhausted(
                "overflow".to_string()
            ))
        );
        assert!(!variables.contains_key("overflow"));
    }

    #[test]
    fn accepts_exactly_the_full_rom_and_rejects_one_more_instruction() {
        let full_rom = vec![Instruction::A(AValue::Number(0)); 32_768];
        assert_eq!(generate(&full_rom).unwrap().len(), 32_768);

        let overflow = vec![Instruction::A(AValue::Number(0)); 32_769];
        assert_eq!(
            generate(&overflow),
            Err(CodegenError::AddressOverflow("32768".to_string()))
        );
    }

    #[test]
    fn allocates_every_variable_address_then_reports_exhaustion() {
        let instructions: Vec<_> = (16..=16_384)
            .map(|address| Instruction::A(AValue::Symbol(format!("variable_{address}"))))
            .collect();

        assert_eq!(
            generate(&instructions),
            Err(CodegenError::VariableAddressSpaceExhausted(
                "variable_16384".to_string()
            ))
        );

        let through_last_address = &instructions[..16_368];
        let output = generate(through_last_address).unwrap();
        assert_eq!(output.last().unwrap(), "0011111111111111");
    }
}
