use crate::{
    parser::VmCommand,
    vm::{Arithmetic, Segment},
};
use hack_assembler::{AValue, Comp, Dest, Instruction, InstructionError, Jump, PredefinedSymbol};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CodegenError {
    #[error("unable to construct generated Hack instruction: {0}")]
    InvalidGeneratedInstruction(#[from] InstructionError),
    #[error("invalid static file stem `{0}`")]
    InvalidStaticStem(String),
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
    symbols: SymbolAllocator,
}

impl Codegen {
    pub fn new(file_stem: impl Into<String>) -> Result<Self, CodegenError> {
        Ok(Self {
            assembly: vec![],
            symbols: SymbolAllocator::new(file_stem.into())?,
        })
    }

    pub fn emit(&mut self, command: &VmCommand) -> Result<(), CodegenError> {
        let assembly_len = self.assembly.len();
        let comparison_count = self.symbols.comparison_count;

        let result = (|| {
            self.assembly
                .push(Instruction::Comment(command.to_string()));
            match command {
                VmCommand::Arithmetic(arithmetic) => {
                    self.generate_arithmetic(arithmetic.clone())?
                }
                VmCommand::Pop { segment, index } => self.generate_pop(segment.clone(), *index)?,
                VmCommand::Push { segment, index } => {
                    self.generate_push(segment.clone(), *index)?
                }
            }
            Ok(())
        })();

        if let Err(error) = result {
            self.assembly.truncate(assembly_len);
            self.symbols.comparison_count = comparison_count;
            return Err(error);
        }
        Ok(())
    }

    pub fn finish(self) -> Vec<Instruction> {
        self.assembly
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
            Arithmetic::Eq => comparison_equality(self.symbols.next_comparison_id())?,
            Arithmetic::Lt => signed_comparison(self.symbols.next_comparison_id(), Jump::Jlt)?,
            Arithmetic::Gt => signed_comparison(self.symbols.next_comparison_id(), Jump::Jgt)?,
        };
        self.assembly.extend_from_slice(&instructions);
        Ok(())
    }

    fn generate_pop(&mut self, segment: Segment, index: u16) -> Result<(), CodegenError> {
        let instructions = match segment {
            Segment::Local => pop_offset(PredefinedSymbol::LCL, index)?,
            Segment::Argument => pop_offset(PredefinedSymbol::ARG, index)?,
            Segment::This => pop_offset(PredefinedSymbol::THIS, index)?,
            Segment::That => pop_offset(PredefinedSymbol::THAT, index)?,
            Segment::Pointer => pop_pointer(index)?,
            Segment::Temp => pop_to(temp_symbol(index)?.into()),
            Segment::Static => pop_to(self.symbols.static_symbol(index)?),
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
            Segment::Static => push_from(self.symbols.static_symbol(index)?),
        };
        self.assembly.extend_from_slice(&instructions);
        Ok(())
    }
}

const INTERNAL_SYMBOL_PREFIX: &str = "__HACK_INTERNALS__";

struct SymbolAllocator {
    static_stem: String,
    comparison_count: usize,
}

impl SymbolAllocator {
    fn new(static_stem: String) -> Result<Self, CodegenError> {
        if static_stem.starts_with(INTERNAL_SYMBOL_PREFIX)
            || Instruction::a_symbol(&static_stem).is_err()
        {
            return Err(CodegenError::InvalidStaticStem(static_stem));
        }
        Ok(Self {
            static_stem,
            comparison_count: 0,
        })
    }

    fn static_symbol(&self, index: u16) -> Result<AValue, CodegenError> {
        let instruction = Instruction::a_symbol(format!("{}.{index}", self.static_stem))?;
        match instruction {
            Instruction::A(value) => Ok(value),
            _ => unreachable!("Instruction::a_symbol always constructs an A-instruction"),
        }
    }

    fn next_comparison_id(&mut self) -> usize {
        let id = self.comparison_count;
        self.comparison_count += 1;
        id
    }

    fn internal_label(label: &str) -> String {
        format!("{INTERNAL_SYMBOL_PREFIX}{label}")
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
fn pop_offset(base: PredefinedSymbol, index: u16) -> Result<Vec<Instruction>, CodegenError> {
    Ok(vec![
        Instruction::a_number(index)?,
        dest_comp(Dest::D, Comp::A),
        base.into(),
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
fn push_offset(base: PredefinedSymbol, index: u16) -> Result<Vec<Instruction>, CodegenError> {
    Ok(vec![
        Instruction::a_number(index)?,
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
    let x_is_neg = format_hack_label(&format!("CMP_X_IS_NEG{}", id));
    let jmp_true = format_hack_label(&format!("CMP_TRUE{}", id));
    let jmp_false = format_hack_label(&format!("CMP_FALSE{}", id));
    let jmp_end = format_hack_label(&format!("CMP_END{}", id));

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
        // @CMP_X_IS_NEG{x}
        // D;JLT
        // -- x is >= 0
        // @R13
        // D=M
        // -- user supplied jump
        // @{CMP_TRUE{x} / CMP_FALSE{x}}
        // D;JLT // x >= 0 && y < 0
        // @SP
        // A=M-1
        // D=M-D
        // @CMP_TRUE{x}
        // -- user supplied comparison
        // D;{JGT | JLT}
        // @CMP_FALSE{x}
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
        // -- x is < 0
        // (CMP_X_IS_NEG{x})
        // @R13
        // D=M
        // -- user supplied arg
        // @{CMP_TRUE{x} / CMP_FALSE{x}}
        // -- x < 0 && y >= 0
        // D;JGE
        // @SP
        // A=M-1
        // D=M-D
        // @CMP_TRUE{x}
        // D;{JGT | JLT} // user supplies this arg
        // @CMP_FALSE{x}
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
        // (CMP_TRUE{x})
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
        // (@CMP_FALSE{x})
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
    let cmp_true_text = format_hack_label(&format!("CMP_TRUE{}", count));
    let cmp_end_text = format_hack_label(&format!("CMP_END{}", count));

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
    SymbolAllocator::internal_label(label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hack_assembler::{InstructionError, format_instructions};

    fn generate(commands: Vec<VmCommand>) -> String {
        let mut codegen = Codegen::new("Test").unwrap();
        for command in &commands {
            codegen.emit(command).unwrap();
        }
        let instructions = codegen
            .finish()
            .into_iter()
            .filter(|instruction| !matches!(instruction, Instruction::Comment(_)))
            .collect::<Vec<_>>();
        format_instructions(&instructions)
    }

    fn generate_with_comments(commands: Vec<VmCommand>) -> String {
        let mut codegen = Codegen::new("Test").unwrap();
        for command in &commands {
            codegen.emit(command).unwrap();
        }
        format_instructions(&codegen.finish())
    }

    #[test]
    fn codegen_uses_structured_hack_instruction_output() {
        let codegen = Codegen::new("Test").unwrap();
        let output: Vec<hack_assembler::Instruction> = codegen.finish();
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
            "@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@__HACK_INTERNALS__CMP_TRUE0\nD;JEQ\n@SP\nA=M-1\nM=0\n@__HACK_INTERNALS__CMP_END0\n0;JMP\n(__HACK_INTERNALS__CMP_TRUE0)\n@SP\nA=M-1\nM=-1\n(__HACK_INTERNALS__CMP_END0)\n"
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
                    "@SP\nAM=M-1\nD=M\n@R13\nM=D\n@SP\nA=M-1\nD=M\n@__HACK_INTERNALS__CMP_X_IS_NEG0\nD;JLT\n@R13\nD=M\n@__HACK_INTERNALS__{nonnegative_x_target}0\nD;JLT\n@SP\nA=M-1\nD=M-D\n@__HACK_INTERNALS__CMP_TRUE0\nD;{comparison_jump}\n@__HACK_INTERNALS__CMP_FALSE0\n0;JMP\n(__HACK_INTERNALS__CMP_X_IS_NEG0)\n@R13\nD=M\n@__HACK_INTERNALS__{negative_x_target}0\nD;JGE\n@SP\nA=M-1\nD=M-D\n@__HACK_INTERNALS__CMP_TRUE0\nD;{comparison_jump}\n@__HACK_INTERNALS__CMP_FALSE0\n0;JMP\n(__HACK_INTERNALS__CMP_TRUE0)\n@SP\nA=M-1\nM=-1\n@__HACK_INTERNALS__CMP_END0\n0;JMP\n(__HACK_INTERNALS__CMP_FALSE0)\n@SP\nA=M-1\nM=0\n(__HACK_INTERNALS__CMP_END0)\n"
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
    fn comparison_labels_remain_unique_across_emit_calls() {
        let mut codegen = Codegen::new("Test").unwrap();
        for arithmetic in [Arithmetic::Eq, Arithmetic::Lt, Arithmetic::Gt] {
            codegen.emit(&VmCommand::Arithmetic(arithmetic)).unwrap();
        }
        let output = format_instructions(&codegen.finish());
        for id in 0..3 {
            assert!(output.contains(&format!("@__HACK_INTERNALS__CMP_TRUE{id}\n")));
            assert!(output.contains(&format!("(__HACK_INTERNALS__CMP_END{id})\n")));
        }
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
        let mut codegen = Codegen::new("Test").unwrap();
        let error = codegen
            .emit(&VmCommand::Pop {
                segment: Segment::Pointer,
                index: 2,
            })
            .unwrap_err();

        assert_eq!(error, CodegenError::InvalidPointerIndex(2));
    }

    #[test]
    fn rejects_out_of_range_offset_addresses() {
        let mut codegen = Codegen::new("Test").unwrap();
        let error = codegen
            .emit(&VmCommand::Pop {
                segment: Segment::Local,
                index: 32_768,
            })
            .unwrap_err();

        assert_eq!(
            error,
            CodegenError::InvalidGeneratedInstruction(InstructionError::AddressOutOfRange(32_768))
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
                .unwrap()
                .emit(&VmCommand::Push {
                    segment: Segment::Temp,
                    index: 8,
                })
                .unwrap_err(),
            CodegenError::InvalidTempIndex(8)
        );
        assert_eq!(
            Codegen::new("Test")
                .unwrap()
                .emit(&VmCommand::Pop {
                    segment: Segment::Constant,
                    index: 0,
                })
                .unwrap_err(),
            CodegenError::CannotPopConstant
        );
        assert_eq!(
            Codegen::new("bad-name").err().unwrap(),
            CodegenError::InvalidStaticStem("bad-name".to_string())
        );
    }

    #[test]
    fn validates_static_stems_before_emitting_output() {
        for stem in ["Foo", "Foo.bar", "Foo$bar", "_Foo", ":Foo"] {
            assert!(Codegen::new(stem).is_ok(), "expected `{stem}` to be valid");
        }
        for stem in [
            "",
            "bad-name",
            "bad name",
            "é",
            "__HACK_INTERNALS__",
            "__HACK_INTERNALS__CMP_TRUE0",
        ] {
            assert_eq!(
                Codegen::new(stem).err().unwrap(),
                CodegenError::InvalidStaticStem(stem.to_string())
            );
        }
    }

    #[test]
    fn static_symbols_cannot_collide_with_internal_labels() {
        assert!(Codegen::new("__HACK_INTERNALS__CMP_TRUE").is_err());

        let output = generate_with_comments(vec![
            VmCommand::Push {
                segment: Segment::Static,
                index: 0,
            },
            VmCommand::Arithmetic(Arithmetic::Eq),
        ]);
        assert!(output.contains("@Test.0\n"));
        assert!(output.contains("(__HACK_INTERNALS__CMP_TRUE0)\n"));
    }

    #[test]
    fn errors_roll_back_partial_output_and_comparison_labels() {
        let mut codegen = Codegen::new("Test").unwrap();
        assert_eq!(
            codegen
                .emit(&VmCommand::Pop {
                    segment: Segment::Pointer,
                    index: 2,
                })
                .unwrap_err(),
            CodegenError::InvalidPointerIndex(2)
        );

        codegen
            .emit(&VmCommand::Arithmetic(Arithmetic::Eq))
            .unwrap();
        let output = format_instructions(&codegen.finish());
        assert!(!output.contains("// pop pointer 2\n"));
        assert!(output.contains("@__HACK_INTERNALS__CMP_TRUE0\n"));
        assert!(!output.contains("@__HACK_INTERNALS__CMP_TRUE1\n"));
    }
}
