use std::collections::HashMap;
use thiserror::Error;

use crate::parser::{AValue, Comp, Dest, Instruction, Jump};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CodegenError {
    #[error("duplicate label `{0}`")]
    DuplicateLabel(String),
    #[error("address space exhausted while resolving `{0}`")]
    AddressOverflow(String),
    #[error("invalid destination `{0}`")]
    InvalidDest(String),
    #[error("invalid computation `{0}`")]
    InvalidComp(String),
    #[error("invalid jump `{0}`")]
    InvalidJump(String),
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
                    comp_bits(comp)?,
                    dest_bits(dest.as_ref())?,
                    jump_bits(jump.as_ref())?
                ));
            }
            Instruction::Label(_) => {}
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
        AValue::Symbol(symbol) => {
            if let Some(address) = labels.get(symbol) {
                return Ok(*address);
            }

            if let Some(address) = variables.get(symbol) {
                return Ok(*address);
            }

            if *next_variable_address == 32_768 {
                return Err(CodegenError::AddressOverflow(symbol.clone()));
            }

            let address = *next_variable_address;
            variables.insert(symbol.clone(), address);
            *next_variable_address += 1;
            Ok(address)
        }
    }
}

fn dest_bits(dest: Option<&Dest>) -> Result<&'static str, CodegenError> {
    match dest.map(Dest::as_str) {
        None => Ok("000"),
        Some("M") => Ok("001"),
        Some("D") => Ok("010"),
        Some("MD") => Ok("011"),
        Some("A") => Ok("100"),
        Some("AM") => Ok("101"),
        Some("AD") => Ok("110"),
        Some("AMD") => Ok("111"),
        Some(value) => Err(CodegenError::InvalidDest(value.to_string())),
    }
}

fn comp_bits(comp: &Comp) -> Result<&'static str, CodegenError> {
    match comp.as_str() {
        "0" => Ok("0101010"),
        "1" => Ok("0111111"),
        "-1" => Ok("0111010"),
        "D" => Ok("0001100"),
        "A" => Ok("0110000"),
        "M" => Ok("1110000"),
        "!D" => Ok("0001101"),
        "!A" => Ok("0110001"),
        "!M" => Ok("1110001"),
        "-D" => Ok("0001111"),
        "-A" => Ok("0110011"),
        "-M" => Ok("1110011"),
        "D+1" => Ok("0011111"),
        "A+1" => Ok("0110111"),
        "M+1" => Ok("1110111"),
        "D-1" => Ok("0001110"),
        "A-1" => Ok("0110010"),
        "M-1" => Ok("1110010"),
        "D+A" => Ok("0000010"),
        "D+M" => Ok("1000010"),
        "D-A" => Ok("0010011"),
        "D-M" => Ok("1010011"),
        "A-D" => Ok("0000111"),
        "M-D" => Ok("1000111"),
        "D&A" => Ok("0000000"),
        "D&M" => Ok("1000000"),
        "D|A" => Ok("0010101"),
        "D|M" => Ok("1010101"),
        value => Err(CodegenError::InvalidComp(value.to_string())),
    }
}

fn jump_bits(jump: Option<&Jump>) -> Result<&'static str, CodegenError> {
    match jump.map(Jump::as_str) {
        None => Ok("000"),
        Some("JGT") => Ok("001"),
        Some("JEQ") => Ok("010"),
        Some("JGE") => Ok("011"),
        Some("JLT") => Ok("100"),
        Some("JNE") => Ok("101"),
        Some("JLE") => Ok("110"),
        Some("JMP") => Ok("111"),
        Some(value) => Err(CodegenError::InvalidJump(value.to_string())),
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
}
