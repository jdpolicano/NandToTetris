use crate::{
    parser::VmCommand,
    vm::{Arithmetic, Segment},
};
use hack_assembler::{AValue, Comp, Dest, Instruction, InstructionError, Jump, PredefinedSymbol};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CodegenError {
    #[error("assembly gen error `{0}`")]
    AssemblyGenError(#[from] InstructionError),
    #[error("invalid pointer index `{0}`")]
    InvalidPointerIndex(u16),
    #[error("invalid temp index `{0}`")]
    InvalidTempIndex(u16),
    #[error("cannot pop into the constant segment")]
    CannotPopConstant,
    #[error("invalid jump for signed comparison {0}")]
    InvalidJumpSignedComparison(Jump),
}

pub struct Codegen {
    assembly: Vec<Instruction>,
    label_count: usize,
    file_name: String,
}

impl Codegen {
    pub fn new(file_name: impl Into<String>) -> Self {
        Self {
            assembly: vec![],
            label_count: 0,
            file_name: file_name.into(),
        }
    }

    pub fn generate(&mut self, commands: Vec<VmCommand>) -> Result<Vec<Instruction>, CodegenError> {
        let assembly_len = self.assembly.len();
        let label_count = self.label_count;

        let result = (|| {
            for command in commands {
                self.assembly
                    .push(Instruction::Comment(command.to_string()));
                match command {
                    VmCommand::Arithmetic(arithmetic) => self.generate_arithmetic(arithmetic)?,
                    VmCommand::Pop { segment, index } => self.generate_pop(segment, index)?,
                    VmCommand::Push { segment, index } => self.generate_push(segment, index)?,
                }
            }
            Ok(())
        })();

        if let Err(error) = result {
            self.assembly.truncate(assembly_len);
            self.label_count = label_count;
            return Err(error);
        }

        Ok(std::mem::take(&mut self.assembly))
    }

    fn get_label_id(&mut self) -> usize {
        let count = self.label_count;
        self.label_count += 1;
        count
    }

    fn generate_arithmetic(&mut self, arithmetic: Arithmetic) -> Result<(), CodegenError> {
        let instructions = match arithmetic {
            // for some x op y (add, sub, and, or)
            Arithmetic::Add => binary_arithmetic(Comp::DPlusM),
            Arithmetic::Sub => binary_arithmetic(Comp::MMinusD),
            Arithmetic::And => binary_arithmetic(Comp::DAndM),
            Arithmetic::Or => binary_arithmetic(Comp::DOrM),
            // for some "op" y (not, neg)
            Arithmetic::Not => unary_arithmetic(Comp::NotM),
            Arithmetic::Neg => unary_arithmetic(Comp::NegM),
            // for some comp (gt, lt, eq)
            Arithmetic::Eq => comparison_equality(self.get_label_id())?,
            Arithmetic::Lt => signed_comparison(self.get_label_id(), Jump::Jlt)?,
            Arithmetic::Gt => signed_comparison(self.get_label_id(), Jump::Jgt)?,
        };
        self.assembly.extend_from_slice(&instructions);
        Ok(())
    }

    fn generate_pop(&mut self, segment: Segment, address: u16) -> Result<(), CodegenError> {
        let instructions = match segment {
            Segment::Local => pop_offset(PredefinedSymbol::LCL, address)?,
            Segment::Argument => pop_offset(PredefinedSymbol::ARG, address)?,
            Segment::This => pop_offset(PredefinedSymbol::THIS, address)?,
            Segment::That => pop_offset(PredefinedSymbol::THAT, address)?,
            Segment::Pointer => pop_pointer(address)?,
            Segment::Temp => pop_to(temp_symbol(address)?.into()),
            Segment::Static => pop_to(static_symbol(&self.file_name, address)?),
            Segment::Constant => return Err(CodegenError::CannotPopConstant),
        };
        self.assembly.extend_from_slice(&instructions);
        Ok(())
    }

    fn generate_push(&mut self, segment: Segment, index: u16) -> Result<(), CodegenError> {
        let instructions = match segment {
            Segment::Constant => push_constant(index)?,
            Segment::Local => push_offset(PredefinedSymbol::LCL, index)?,
            Segment::Argument => push_offset(PredefinedSymbol::ARG, index)?,
            Segment::This => push_offset(PredefinedSymbol::THIS, index)?,
            Segment::That => push_offset(PredefinedSymbol::THAT, index)?,
            Segment::Pointer => push_from(pointer_symbol(index)?.into()),
            Segment::Temp => push_from(temp_symbol(index)?.into()),
            Segment::Static => push_from(static_symbol(&self.file_name, index)?),
        };
        self.assembly.extend_from_slice(&instructions);
        Ok(())
    }
}

// @offset
// D=A        // D = offset

// @LCL
// D=D+M      // D = offset + base

// @R13
// M=D        // R13 holds the target address

// @SP
// AM=M-1     // SP--, A points to popped value
// D=M        // D = popped value

// @R13
// A=M        // A = target address
// M=D        // *target = popped value
fn pop_offset(target: PredefinedSymbol, offset: u16) -> Result<Vec<Instruction>, CodegenError> {
    Ok(vec![
        Instruction::a_number(offset)?,
        dest_comp(Dest::D, Comp::A),
        target.into(),
        dest_comp(Dest::D, Comp::DPlusM),
        PredefinedSymbol::R13.into(),
        dest_comp(Dest::M, Comp::D),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::AM, Comp::MMinusOne),
        dest_comp(Dest::D, Comp::M),
        PredefinedSymbol::R13.into(),
        dest_comp(Dest::A, Comp::M),
        dest_comp(Dest::M, Comp::D),
    ])
}

// pops the value from the stack and puts it in either "this" or "that"
// @SP
// AM=M-1
// D=M
// @THIS // or THAT
// M=D
fn pop_pointer(index: u16) -> Result<Vec<Instruction>, CodegenError> {
    Ok(pop_to(pointer_symbol(index)?.into()))
}

// Pushes a literal value.
// @value
// D=A
// @SP
// A=M        // A points to the next stack slot
// M=D
// @SP
// M=M+1
fn push_constant(value: u16) -> Result<Vec<Instruction>, CodegenError> {
    Ok(vec![
        Instruction::a_number(value)?,
        dest_comp(Dest::D, Comp::A),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::M),
        dest_comp(Dest::M, Comp::D),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::M, Comp::MPlusOne),
    ])
}

// Pushes base[offset] for local, argument, this, or that.
// @offset
// D=A
// @base
// A=D+M      // A points to base + offset
// D=M        // D holds base[offset]
// @SP
// A=M
// M=D
// @SP
// M=M+1
fn push_offset(base: PredefinedSymbol, offset: u16) -> Result<Vec<Instruction>, CodegenError> {
    Ok(vec![
        Instruction::a_number(offset)?,
        dest_comp(Dest::D, Comp::A),
        base.into(),
        dest_comp(Dest::A, Comp::DPlusM),
        dest_comp(Dest::D, Comp::M),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::M),
        dest_comp(Dest::M, Comp::D),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::M, Comp::MPlusOne),
    ])
}

// Pushes the value stored at an A-instruction operand.
// @source
// D=M
// @SP
// A=M
// M=D
// @SP
// M=M+1
fn push_from(source: AValue) -> Vec<Instruction> {
    vec![
        Instruction::A(source),
        dest_comp(Dest::D, Comp::M),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::M),
        dest_comp(Dest::M, Comp::D),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::M, Comp::MPlusOne),
    ]
}

// Pops the top stack value into an A-instruction operand.
// @SP
// AM=M-1
// D=M
// @target
// M=D
fn pop_to(target: AValue) -> Vec<Instruction> {
    vec![
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::AM, Comp::MMinusOne),
        dest_comp(Dest::D, Comp::M),
        Instruction::A(target),
        dest_comp(Dest::M, Comp::D),
    ]
}

fn pointer_symbol(index: u16) -> Result<PredefinedSymbol, CodegenError> {
    match index {
        0 => Ok(PredefinedSymbol::THIS),
        1 => Ok(PredefinedSymbol::THAT),
        _ => Err(CodegenError::InvalidPointerIndex(index)),
    }
}

fn temp_symbol(index: u16) -> Result<PredefinedSymbol, CodegenError> {
    match index {
        0 => Ok(PredefinedSymbol::R5),
        1 => Ok(PredefinedSymbol::R6),
        2 => Ok(PredefinedSymbol::R7),
        3 => Ok(PredefinedSymbol::R8),
        4 => Ok(PredefinedSymbol::R9),
        5 => Ok(PredefinedSymbol::R10),
        6 => Ok(PredefinedSymbol::R11),
        7 => Ok(PredefinedSymbol::R12),
        _ => Err(CodegenError::InvalidTempIndex(index)),
    }
}

fn static_symbol(file_name: &str, index: u16) -> Result<AValue, CodegenError> {
    match Instruction::a_symbol(format!("{file_name}.{index}"))? {
        Instruction::A(value) => Ok(value),
        _ => unreachable!("Instruction::a_symbol always constructs an A-instruction"),
    }
}

// for some x op y (add, sub, and, or)
// @SP
// AM=M-1   // SP--, A points to y
// D=M      // D = y
// A=A-1    // A points to x
// M=M op D    // x = x - y or x + y or x & y or x | y
fn binary_arithmetic(op: Comp) -> Vec<Instruction> {
    vec![
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::AM, Comp::MMinusOne),
        dest_comp(Dest::D, Comp::M),
        dest_comp(Dest::A, Comp::AMinusOne),
        dest_comp(Dest::M, op),
    ]
}

// for some "op" y (not, neg)
// @SP
// A=M-1   // A points to y, write over it as is.
// M="op"+ M
fn unary_arithmetic(op: Comp) -> Vec<Instruction> {
    vec![
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::MMinusOne),
        dest_comp(Dest::M, op),
    ]
}

/*
* perfoms x comp y and handles overflow issues
*/
fn signed_comparison(id: usize, comparison: Jump) -> Result<Vec<Instruction>, CodegenError> {
    let x_is_neg = format_hack_label(&format!("CMP_X_IS_NEG.{}", id));
    let jmp_true = format_hack_label(&format!("CMP_TRUE.{}", id));
    let jmp_false = format_hack_label(&format!("CMP_FALSE.{}", id));
    let jmp_end = format_hack_label(&format!("CMP_END.{}", id));

    let (x_gt_y_target, x_lt_y_target) = match comparison {
        Jump::Jgt => (jmp_true.clone(), jmp_false.clone()),
        Jump::Jlt => (jmp_false.clone(), jmp_true.clone()),
        _ => return Err(CodegenError::InvalidJumpSignedComparison(comparison)),
    };

    Ok(vec![
        // perfoms x comp y and handles overflow issues
        // -- store y
        // @SP
        // AM=M-1
        // D=M
        // @R13
        // M=D
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::AM, Comp::MMinusOne),
        dest_comp(Dest::D, Comp::M),
        PredefinedSymbol::R13.into(),
        dest_comp(Dest::M, Comp::D),
        // -- determine if x is negative
        // @SP
        // A=M-1
        // D=M
        // @CMP_X_IS_NEG.{x}
        // D;JLT
        // // x >= 0
        // @R13
        // D=M
        // @{CMP_TRUE.{x} / CMP_FALSE.{x}} // user supplies this arg
        // D;JLT // x >= 0 && y < 0
        // @SP
        // A=M-1
        // D=M-D
        // @CMP_TRUE.{x}
        // D;{JGT | JLT}
        // @CMP_FALSE.{x}
        // 0;JMP
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::MMinusOne),
        dest_comp(Dest::D, Comp::M),
        Instruction::a_symbol(x_is_neg.clone())?,
        comp_jump(Comp::D, Jump::Jlt),
        PredefinedSymbol::R13.into(),
        dest_comp(Dest::D, Comp::M),
        Instruction::a_symbol(x_gt_y_target)?,
        comp_jump(Comp::D, Jump::Jlt),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::MMinusOne),
        dest_comp(Dest::D, Comp::MMinusD),
        Instruction::a_symbol(jmp_true.clone())?,
        comp_jump(Comp::D, comparison),
        Instruction::a_symbol(jmp_false.clone())?,
        comp_jump(Comp::Zero, Jump::Jmp),
        // (CMP_X_IS_NEG.{x})
        // // x < 0
        // @R13
        // D=M
        // @{CMP_TRUE.{x} / CMP_FALSE.{x}} // user supplies this arg
        // D;JGE // x < 0 && y >= 0
        // @SP
        // A=M-1
        // D=M-D
        // @CMP_TRUE.{x}
        // D;{JGT | JLT} // user supplies this arg
        // @CMP_FALSE.{x}
        // 0;JMP
        Instruction::label(x_is_neg)?,
        PredefinedSymbol::R13.into(),
        dest_comp(Dest::D, Comp::M),
        Instruction::a_symbol(x_lt_y_target)?,
        comp_jump(Comp::D, Jump::Jge),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::MMinusOne),
        dest_comp(Dest::D, Comp::MMinusD),
        Instruction::a_symbol(jmp_true.clone())?,
        comp_jump(Comp::D, comparison),
        Instruction::a_symbol(jmp_false.clone())?,
        comp_jump(Comp::Zero, Jump::Jmp),
        // (CMP_TRUE.{x})
        // @SP
        // A=M-1
        // M=-1
        // @CMP_END
        // 0;JMP
        Instruction::label(jmp_true)?,
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::MMinusOne),
        dest_comp(Dest::M, Comp::NegOne),
        Instruction::a_symbol(jmp_end.clone())?,
        comp_jump(Comp::Zero, Jump::Jmp),
        // (@CMP_FALSE.{x})
        // @SP
        // A=M-1
        // M=0
        Instruction::label(jmp_false)?,
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::MMinusOne),
        dest_comp(Dest::M, Comp::Zero),
        // (CMP_END)
        Instruction::label(jmp_end)?,
    ])
}

fn comparison_equality(count: usize) -> Result<Vec<Instruction>, CodegenError> {
    let cmp_true_text = format_hack_label(&format!("CMP_TRUE.{}", count));
    let cmp_end_text = format_hack_label(&format!("CMP_END.{}", count));

    Ok(vec![
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::AM, Comp::MMinusOne),
        dest_comp(Dest::D, Comp::M),
        dest_comp(Dest::A, Comp::AMinusOne),
        dest_comp(Dest::D, Comp::MMinusD),
        Instruction::a_symbol(cmp_true_text.clone())?,
        comp_jump(Comp::D, Jump::Jeq),
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::MMinusOne),
        dest_comp(Dest::M, Comp::Zero),
        Instruction::a_symbol(cmp_end_text.clone())?,
        comp_jump(Comp::Zero, Jump::Jmp),
        Instruction::label(cmp_true_text)?,
        PredefinedSymbol::SP.into(),
        dest_comp(Dest::A, Comp::MMinusOne),
        dest_comp(Dest::M, Comp::NegOne),
        Instruction::label(cmp_end_text)?,
    ])
}

fn dest_comp(dest: Dest, comp: Comp) -> Instruction {
    Instruction::C {
        dest: Some(dest),
        comp,
        jump: None,
    }
}

fn comp_jump(comp: Comp, jump: Jump) -> Instruction {
    Instruction::C {
        dest: None,
        comp,
        jump: Some(jump),
    }
}

fn format_hack_label(label: &str) -> String {
    format!("__HACK_INTERNALS__.{}", label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hack_assembler::{InstructionError, format_instructions};

    fn generate(commands: Vec<VmCommand>) -> String {
        let instructions = Codegen::new("Test")
            .generate(commands)
            .unwrap()
            .into_iter()
            .filter(|instruction| !matches!(instruction, Instruction::Comment(_)))
            .collect::<Vec<_>>();
        format_instructions(&instructions)
    }

    fn generate_with_comments(commands: Vec<VmCommand>) -> String {
        format_instructions(&Codegen::new("Test").generate(commands).unwrap())
    }

    #[test]
    fn codegen_uses_structured_hack_instruction_output() {
        let mut codegen = Codegen::new("Test");
        let output: Vec<hack_assembler::Instruction> = codegen.generate(vec![]).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn emits_one_source_comment_before_each_vm_command_section() {
        assert_eq!(
            generate_with_comments(vec![
                VmCommand::Push {
                    segment: Segment::Constant,
                    index: 7,
                },
                VmCommand::Arithmetic(Arithmetic::Neg),
            ]),
            "// push constant 7\n@7\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n// neg\n@SP\nA=M-1\nM=-M\n"
        );
    }

    #[test]
    fn generates_binary_arithmetic() {
        let cases = [
            (Arithmetic::Add, "D+M"),
            (Arithmetic::Sub, "M-D"),
            (Arithmetic::And, "D&M"),
            (Arithmetic::Or, "D|M"),
        ];

        for (arithmetic, operation) in cases {
            assert_eq!(
                generate(vec![VmCommand::Arithmetic(arithmetic)]),
                format!("@SP\nAM=M-1\nD=M\nA=A-1\nM={operation}\n")
            );
        }
    }

    #[test]
    fn generates_unary_arithmetic() {
        let cases = [(Arithmetic::Neg, "-M"), (Arithmetic::Not, "!M")];

        for (arithmetic, operation) in cases {
            assert_eq!(
                generate(vec![VmCommand::Arithmetic(arithmetic)]),
                format!("@SP\nA=M-1\nM={operation}\n")
            );
        }
    }

    #[test]
    fn generates_equality_comparison() {
        assert_eq!(
            generate(vec![VmCommand::Arithmetic(Arithmetic::Eq)]),
            "@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@__HACK_INTERNALS__.CMP_TRUE.0\nD;JEQ\n@SP\nA=M-1\nM=0\n@__HACK_INTERNALS__.CMP_END.0\n0;JMP\n(__HACK_INTERNALS__.CMP_TRUE.0)\n@SP\nA=M-1\nM=-1\n(__HACK_INTERNALS__.CMP_END.0)\n"
        );
    }

    #[test]
    fn generates_overflow_safe_signed_comparisons() {
        let cases = [
            (Arithmetic::Gt, "JGT", "CMP_TRUE", "CMP_FALSE"),
            (Arithmetic::Lt, "JLT", "CMP_FALSE", "CMP_TRUE"),
        ];

        for (arithmetic, comparison_jump, nonnegative_x_target, negative_x_target) in cases {
            assert_eq!(
                generate(vec![VmCommand::Arithmetic(arithmetic)]),
                format!(
                    "@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@__HACK_INTERNALS__.CMP_X_IS_NEG.0\nD;JLT\n@R13\nD=M\n@__HACK_INTERNALS__.{nonnegative_x_target}.0\nD;JLT\n@SP\nA=M-1\nD=M-D\n@__HACK_INTERNALS__.CMP_TRUE.0\nD;{comparison_jump}\n@__HACK_INTERNALS__.CMP_FALSE.0\n0;JMP\n(__HACK_INTERNALS__.CMP_X_IS_NEG.0)\n@R13\nD=M\n@__HACK_INTERNALS__.{negative_x_target}.0\nD;JGE\n@SP\nA=M-1\nD=M-D\n@__HACK_INTERNALS__.CMP_TRUE.0\nD;{comparison_jump}\n@__HACK_INTERNALS__.CMP_FALSE.0\n0;JMP\n(__HACK_INTERNALS__.CMP_TRUE.0)\n@SP\nA=M-1\nM=-1\n@__HACK_INTERNALS__.CMP_END.0\n0;JMP\n(__HACK_INTERNALS__.CMP_FALSE.0)\n@SP\nA=M-1\nM=0\n(__HACK_INTERNALS__.CMP_END.0)\n"
                )
            );
        }
    }

    #[test]
    fn signed_comparison_rejects_non_ordering_jumps() {
        assert_eq!(
            signed_comparison(0, Jump::Jeq),
            Err(CodegenError::InvalidJumpSignedComparison(Jump::Jeq))
        );
    }

    #[test]
    fn comparison_labels_remain_unique_across_generate_calls() {
        let mut codegen = Codegen::new("Test");

        let first = codegen
            .generate(vec![
                VmCommand::Arithmetic(Arithmetic::Eq),
                VmCommand::Arithmetic(Arithmetic::Lt),
            ])
            .unwrap();
        let second = codegen
            .generate(vec![VmCommand::Arithmetic(Arithmetic::Gt)])
            .unwrap();

        let first = format_instructions(&first);
        let second = format_instructions(&second);
        assert!(first.contains("@__HACK_INTERNALS__.CMP_TRUE.0\n"));
        assert!(first.contains("(__HACK_INTERNALS__.CMP_END.0)\n"));
        assert!(first.contains("@__HACK_INTERNALS__.CMP_TRUE.1\n"));
        assert!(first.contains("(__HACK_INTERNALS__.CMP_END.1)\n"));
        assert!(second.contains("@__HACK_INTERNALS__.CMP_TRUE.2\n"));
        assert!(second.contains("(__HACK_INTERNALS__.CMP_END.2)\n"));
    }

    #[test]
    fn generates_offset_segment_pops() {
        let cases = [
            (Segment::Local, "LCL"),
            (Segment::Argument, "ARG"),
            (Segment::This, "THIS"),
            (Segment::That, "THAT"),
        ];

        for (segment, base) in cases {
            assert_eq!(
                generate(vec![VmCommand::Pop { segment, index: 7 }]),
                format!("@7\nD=A\n@{base}\nD=D+M\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n")
            );
        }
    }

    #[test]
    fn generates_pointer_pops() {
        for (index, target) in [(0, "THIS"), (1, "THAT")] {
            assert_eq!(
                generate(vec![VmCommand::Pop {
                    segment: Segment::Pointer,
                    index,
                }]),
                format!("@SP\nAM=M-1\nD=M\n@{target}\nM=D\n")
            );
        }
    }

    #[test]
    fn concatenates_commands_in_input_order() {
        assert_eq!(
            generate(vec![
                VmCommand::Arithmetic(Arithmetic::Neg),
                VmCommand::Pop {
                    segment: Segment::Pointer,
                    index: 0,
                },
            ]),
            "@SP\nA=M-1\nM=-M\n@SP\nAM=M-1\nD=M\n@THIS\nM=D\n"
        );
    }

    #[test]
    fn rejects_invalid_pointer_indices() {
        let error = Codegen::new("Test")
            .generate(vec![VmCommand::Pop {
                segment: Segment::Pointer,
                index: 2,
            }])
            .unwrap_err();

        assert_eq!(error, CodegenError::InvalidPointerIndex(2));
    }

    #[test]
    fn rejects_out_of_range_offset_addresses() {
        let error = Codegen::new("Test")
            .generate(vec![VmCommand::Pop {
                segment: Segment::Local,
                index: 32_768,
            }])
            .unwrap_err();

        assert_eq!(
            error,
            CodegenError::AssemblyGenError(InstructionError::AddressOutOfRange(32_768))
        );
    }

    #[test]
    fn generates_constant_pushes() {
        assert_eq!(
            generate(vec![VmCommand::Push {
                segment: Segment::Constant,
                index: 7,
            }]),
            "@7\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"
        );
    }

    #[test]
    fn generates_offset_segment_pushes() {
        for (segment, base) in [
            (Segment::Local, "LCL"),
            (Segment::Argument, "ARG"),
            (Segment::This, "THIS"),
            (Segment::That, "THAT"),
        ] {
            assert_eq!(
                generate(vec![VmCommand::Push { segment, index: 7 }]),
                format!("@7\nD=A\n@{base}\nA=D+M\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n")
            );
        }
    }

    #[test]
    fn generates_pointer_temp_and_static_accesses() {
        let cases = [
            (Segment::Pointer, 1, "THAT"),
            (Segment::Temp, 7, "R12"),
            (Segment::Static, 3, "Test.3"),
        ];

        for (segment, index, symbol) in cases {
            assert_eq!(
                generate(vec![VmCommand::Push { segment, index }]),
                format!("@{symbol}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n")
            );
        }

        for (segment, index, symbol) in [(Segment::Temp, 0, "R5"), (Segment::Static, 3, "Test.3")] {
            assert_eq!(
                generate(vec![VmCommand::Pop { segment, index }]),
                format!("@SP\nAM=M-1\nD=M\n@{symbol}\nM=D\n")
            );
        }
    }

    #[test]
    fn rejects_invalid_memory_accesses() {
        assert_eq!(
            Codegen::new("Test")
                .generate(vec![VmCommand::Push {
                    segment: Segment::Temp,
                    index: 8,
                }])
                .unwrap_err(),
            CodegenError::InvalidTempIndex(8)
        );
        assert_eq!(
            Codegen::new("Test")
                .generate(vec![VmCommand::Pop {
                    segment: Segment::Constant,
                    index: 0,
                }])
                .unwrap_err(),
            CodegenError::CannotPopConstant
        );
        assert_eq!(
            Codegen::new("bad-name")
                .generate(vec![VmCommand::Push {
                    segment: Segment::Static,
                    index: 0,
                }])
                .unwrap_err(),
            CodegenError::AssemblyGenError(InstructionError::InvalidSymbol(
                "bad-name.0".to_string()
            ))
        );
    }

    #[test]
    fn errors_roll_back_partial_output_and_comparison_labels() {
        let mut codegen = Codegen::new("Test");
        assert_eq!(
            codegen
                .generate(vec![
                    VmCommand::Arithmetic(Arithmetic::Eq),
                    VmCommand::Pop {
                        segment: Segment::Pointer,
                        index: 2,
                    },
                ])
                .unwrap_err(),
            CodegenError::InvalidPointerIndex(2)
        );

        let output = format_instructions(
            &codegen
                .generate(vec![VmCommand::Arithmetic(Arithmetic::Eq)])
                .unwrap(),
        );
        assert!(output.contains("@__HACK_INTERNALS__.CMP_TRUE.0\n"));
        assert!(!output.contains("@__HACK_INTERNALS__.CMP_TRUE.1\n"));
    }
}
