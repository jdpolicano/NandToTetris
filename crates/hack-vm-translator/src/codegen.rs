use crate::parser::{ParseError, Parser, VmCommand};
use crate::vm::{Arithmetic, Segment};

pub struct CodeGen {
    assembly: Vec<String>,
}

impl CodeGen {
    pub fn new() -> Self {
        Self { assembly: vec![] }
    }

    fn gen_inc_stack_pointer(&mut self) {
        self.assembly.extend(["@sp", "m=m+1"])
    }

    fn gen_arithmetic(&mut self, op: Arithmetic) {
        match op {
            Arithmetic::Add => {}
            _ => {}
        }
    }

    pub fn generate(&mut self, commands: Vec<VmCommand>) -> Vec<String> {
        let mut assembly = Vec::new();
        for command in commands {
            match command {
                VmCommand::Arithmetic(arithmetic) => {}
                VmCommand::Pop { segment, index } => {}
                VmCommand::Push { segment, index } => {}
            }
        }
        assembly
    }
}
