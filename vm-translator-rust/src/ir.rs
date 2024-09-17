use crate::parser::VmCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comp {
    Zero,
    NegOne,
    One,
    Data,
    Mem,
    Addr,
    NotData,
    NotMem,
    NotAddr,
    NegData,
    NegMem,
    NegAddr,
    DataPlusOne,
    MemPlusOne,
    AddrPlusOne,
    DataMinusOne,
    MemMinusOne,
    AddrMinusOne,
    DataPlusAddr,
    DataPlusMem,
    DataMinusAddr,
    AddrMinusData,
    MemMinusData,
    DataAndAddr,
    DataAndMem,
    DataOrAddr,
    DataOrMem,
}

impl Comp {
    pub fn as_str(&self) -> &'static str {
        match self {
            Comp::Zero => "0",
            Comp::NegOne => "-1",
            Comp::One => "1",
            Comp::Data => "D",
            Comp::Mem => "M",
            Comp::Addr => "A",
            Comp::NotData => "!D",
            Comp::NotMem => "!M",
            Comp::NotAddr => "!A",
            Comp::NegData => "-D",
            Comp::NegMem => "-M",
            Comp::NegAddr => "-A",
            Comp::DataPlusOne => "D+1",
            Comp::MemPlusOne => "M+1",
            Comp::AddrPlusOne => "A+1",
            Comp::DataMinusOne => "D-1",
            Comp::MemMinusOne => "M-1",
            Comp::AddrMinusOne => "A-1",
            Comp::DataPlusAddr => "D+A",
            Comp::DataPlusMem => "D+M",
            Comp::DataMinusAddr => "D-A",
            Comp::AddrMinusData => "A-D",
            Comp::MemMinusData => "M-D",
            Comp::DataAndAddr => "D&A",
            Comp::DataAndMem => "D&M",
            Comp::DataOrAddr => "D|A",
            Comp::DataOrMem => "D|M",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dest {
    Mem,
    Data,
    DataMem,
    Addr,
    AddrMem,
    AddrData,
    All,
}

impl Dest {
    pub fn as_str(&self) -> &'static str {
        match self {
            Dest::Mem => "M",
            Dest::Data => "D",
            Dest::DataMem => "DM",
            Dest::Addr => "A",
            Dest::AddrMem => "AM",
            Dest::AddrData => "AD",
            Dest::All => "ADM",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jump {
    Jgt,
    Jeq,
    Jge,
    Jlt,
    Jne,
    Jle,
    Jmp,
}

impl Jump {
    pub fn as_str(&self) -> &'static str {
        match self {
            Jump::Jgt => "JGT",
            Jump::Jeq => "JEQ",
            Jump::Jge => "JGE",
            Jump::Jlt => "JLT",
            Jump::Jne => "JNE",
            Jump::Jle => "JLE",
            Jump::Jmp => "JMP",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsmIr {
    Push,                     // Push D to stack then Sp++
    Pop,                      // Pop from stack to D then Sp--
    DecAssign(Dest, Comp),    // A = A - 1; Dest = Comp
    TopAssign(Dest, Comp),    // A = Sp - 1; Dest = Comp
    LoadOffset(String, u16),  // Load offset of a segment to data
    LoadConstant(u16),        // Load constant to data
    LoadAddress(String),      // Load an Address's mem to data
    WriteToAddress(String),   // Load Data to an Address
    StoreOffset(String, u16), // Store the address of an offset of a segment to a temp register
    DerefWrite(String),       // dereference the address, then write
    Jump(String, Jump),       // Jump to label with condition on Data
    Label(String),            // Label
    Assign(Dest, Comp),       // Dest = Comp - this will be used in optimization steps,
    Address(String),          // @Address
    Comment(String),          // Comments
}

impl AsmIr {
    pub fn to_string(self) -> String {
        match self {
            AsmIr::Push => Self::push(),
            AsmIr::Pop => Self::pop(),
            AsmIr::DecAssign(dest, comp) => Self::dec_assign(dest, comp),
            AsmIr::TopAssign(dest, comp) => Self::top_assign(dest, comp),
            AsmIr::LoadOffset(segment, offset) => Self::load_offset(segment, offset),
            AsmIr::LoadConstant(value) => Self::load_constant(value),
            AsmIr::LoadAddress(address) => Self::load_address(address),
            AsmIr::WriteToAddress(address) => Self::write_to_address(address),
            AsmIr::StoreOffset(segment, offset) => Self::store_offset(segment, offset),
            AsmIr::DerefWrite(address) => Self::deref_write(address),
            AsmIr::Jump(label, jump) => Self::jump(label, jump),
            AsmIr::Label(label) => Self::label(label),
            AsmIr::Assign(dest, comp) => Self::assign(dest, comp),
            AsmIr::Address(address) => Self::address(address),
            AsmIr::Comment(comment) => comment,
        }
    }

    fn push() -> String {
        "@SP\nA=M\nM=D\n@SP\nM=M+1\n".to_string()
    }

    fn pop() -> String {
        "@SP\nAM=M-1\nD=M\n".to_string()
    }

    fn dec_assign(dest: Dest, comp: Comp) -> String {
        format!("A=A-1\n{}={}\n", dest.as_str(), comp.as_str())
    }

    fn top_assign(dest: Dest, comp: Comp) -> String {
        format!("@SP\nA=M-1\n{}={}\n", dest.as_str(), comp.as_str())
    }

    fn load_offset(segment: String, offset: u16) -> String {
        format!("@{}\nD=A\n@{}\nA=D+M\nD=M\n", offset, segment)
    }

    fn load_constant(value: u16) -> String {
        format!("@{}\nD=A\n", value)
    }

    fn load_address(address: String) -> String {
        format!("@{}\nD=M\n", address)
    }

    fn write_to_address(address: String) -> String {
        format!("@{}\nM=D\n", address)
    }

    fn store_offset(segment: String, offset: u16) -> String {
        format!("@{}\nD=A\n@{}\nD=D+M\n@R13\nM=D\n", offset, segment)
    }

    pub fn deref_write(address: String) -> String {
        format!("@{}\nA=M\nM=D\n", address)
    }

    fn jump(label: String, jump: Jump) -> String {
        format!("@{}\nD;{}\n", label, jump.as_str())
    }

    fn label(label: String) -> String {
        format!("({})\n", label)
    }

    fn assign(dest: Dest, comp: Comp) -> String {
        format!("{}={}\n", dest.as_str(), comp.as_str())
    }

    fn address(address: String) -> String {
        format!("@{}\n", address)
    }
}

pub struct IrParser<'a> {
    pub commands: Vec<AsmIr>,
    conditional_counter: u16,
    filename: &'a str,
}

impl<'a> IrParser<'a> {
    pub fn new(filename: &'a str) -> Self {
        IrParser {
            commands: Vec::new(),
            conditional_counter: 0,
            filename,
        }
    }

    pub fn parse(&mut self, command: VmCommand) -> Result<(), String> {
        match command {
            VmCommand::Add => {
                self.comment("Add");
                self.commands.push(AsmIr::Pop);
                self.commands
                    .push(AsmIr::DecAssign(Dest::Mem, Comp::DataPlusMem));
                Ok(())
            }

            VmCommand::Sub => {
                self.comment("Sub");
                self.commands.push(AsmIr::Pop);
                self.commands
                    .push(AsmIr::DecAssign(Dest::Mem, Comp::MemMinusData));
                Ok(())
            }

            VmCommand::And => {
                self.comment("And");
                self.commands.push(AsmIr::Pop);
                self.commands
                    .push(AsmIr::DecAssign(Dest::Mem, Comp::DataAndMem));
                Ok(())
            }

            VmCommand::Or => {
                self.comment("Or");
                self.commands.push(AsmIr::Pop);
                self.commands
                    .push(AsmIr::DecAssign(Dest::Mem, Comp::DataOrMem));
                Ok(())
            }
            VmCommand::Neg => {
                self.comment("Neg");
                self.commands
                    .push(AsmIr::TopAssign(Dest::Mem, Comp::NegMem));
                Ok(())
            }
            VmCommand::Not => {
                self.comment("Not");
                self.commands
                    .push(AsmIr::TopAssign(Dest::Mem, Comp::NotMem));
                Ok(())
            }
            VmCommand::Gt => self.push_comparison("JGT", Jump::Jgt),
            VmCommand::Lt => self.push_comparison("JLT", Jump::Jlt),
            VmCommand::Eq => self.push_comparison("JEQ", Jump::Jeq),
            VmCommand::Push { segment, index } => self.push(segment, index),
            VmCommand::Pop { segment, index } => self.pop(segment, index),
            _ => Err("Not implemented".to_string()),
        }
    }

    /// this is an experimental step I'm taking to try and see if we can reduce the instruction count on these programs
    /// by optimizing the generated assembly code
    /// This one optimization reduced the stacktest.vm program from 321 instructions to 249 and still passes the tests
    /// If this holds across the board we may see a 20% performance improvement across all assembly!
    ///
    /// Known issues:
    /// Currently the comments appear completely out of order unfortunately. I'm not sure how to fix this yet.
    ///
    pub fn optimize(&mut self) {
        let mut optimized: Vec<AsmIr> = Vec::new();
        let mut iter = self.commands.iter().peekable();
        while let Some(ir) = iter.next() {
            if let Some(mut next) = iter.peek() {
                let mut comment = None;
                match next {
                    AsmIr::Comment(_) => {
                        println!("comment found!");
                        comment = iter.next();
                        next = iter.peek().unwrap();
                    }
                    _ => {}
                };

                match (ir, next) {
                    (AsmIr::Push, AsmIr::Pop) => {
                        println!("Optimizing push/pop");
                        iter.next();
                        optimized.push(AsmIr::Address("SP".to_string()));
                        optimized.push(AsmIr::Assign(Dest::Addr, Comp::Mem));
                        if comment.is_some() {
                            optimized.push(comment.unwrap().clone());
                        }
                    }
                    _ => {
                        optimized.push(ir.clone());
                        if comment.is_some() {
                            optimized.push(comment.unwrap().clone());
                        }
                    }
                }
            } else {
                optimized.push(ir.clone());
            }
        }
        self.commands = optimized;
    }

    fn push_comparison(&mut self, condition: &str, jmp: Jump) -> Result<(), String> {
        self.comment(&format!("Comparison {}", condition));
        let if_true = format!(
            "{}_TRUE_{}.{}",
            condition, self.filename, self.conditional_counter
        );

        let end = format!(
            "{}_END_{}.{}",
            condition, self.filename, self.conditional_counter
        );

        self.conditional_counter += 1;
        self.commands.push(AsmIr::Pop);
        self.commands
            .push(AsmIr::DecAssign(Dest::Data, Comp::MemMinusData));
        self.commands.push(AsmIr::Jump(if_true.clone(), jmp));
        self.commands.push(AsmIr::TopAssign(Dest::Mem, Comp::Zero));
        self.commands.push(AsmIr::Jump(end.clone(), Jump::Jmp));
        self.commands.push(AsmIr::Label(if_true));
        self.commands
            .push(AsmIr::TopAssign(Dest::Mem, Comp::NegOne));
        self.commands.push(AsmIr::Label(end));
        Ok(())
    }

    fn push(&mut self, segment: &str, index: u16) -> Result<(), String> {
        self.comment(&format!("Push {} {}", segment, index));
        match segment {
            "constant" => {
                self.commands.push(AsmIr::LoadConstant(index));
                self.commands.push(AsmIr::Push);
                Ok(())
            }

            "static" => {
                self.commands
                    .push(AsmIr::LoadAddress(format!("{}.{}", self.filename, index)));
                self.commands.push(AsmIr::Push);
                Ok(())
            }

            "pointer" if index == 0 => {
                self.commands.push(AsmIr::LoadAddress("THIS".to_string()));
                self.commands.push(AsmIr::Push);
                Ok(())
            }

            "pointer" if index == 1 => {
                self.commands.push(AsmIr::LoadAddress("THAT".to_string()));
                self.commands.push(AsmIr::Push);
                Ok(())
            }
            "temp" => {
                if index > 7 {
                    return Err(format!("Invalid index for push temp, received {}", index));
                }
                self.commands
                    .push(AsmIr::LoadAddress(format!("R{}", 5 + index)));
                self.commands.push(AsmIr::Push);
                Ok(())
            }
            _ => {
                let reference = self.resolve_reference(segment)?;
                self.commands.push(AsmIr::LoadOffset(reference, index));
                self.commands.push(AsmIr::Push);
                Ok(())
            }
        }
    }

    fn pop(&mut self, segment: &str, index: u16) -> Result<(), String> {
        self.comment(&format!("Pop {} {}", segment, index));
        match segment {
            "static" => {
                self.commands.push(AsmIr::Pop);
                self.commands.push(AsmIr::WriteToAddress(format!(
                    "{}.{}",
                    self.filename, index
                )));
                Ok(())
            }

            "pointer" if index == 0 => {
                self.commands.push(AsmIr::Pop);
                self.commands
                    .push(AsmIr::WriteToAddress("THIS".to_string()));
                Ok(())
            }

            "pointer" if index == 1 => {
                self.commands.push(AsmIr::Pop);
                self.commands
                    .push(AsmIr::WriteToAddress("THAT".to_string()));
                Ok(())
            }
            "temp" => {
                if index > 7 {
                    return Err(format!("Invalid index for pop temp, received {}", index));
                }
                self.commands.push(AsmIr::Pop);
                self.commands
                    .push(AsmIr::WriteToAddress(format!("R{}", 5 + index)));
                Ok(())
            }
            _ => {
                let reference = self.resolve_reference(segment)?;
                self.commands.push(AsmIr::StoreOffset(reference, index)); // store the computed address to R13
                self.commands.push(AsmIr::Pop);
                self.commands.push(AsmIr::DerefWrite("R13".to_string()));
                Ok(())
            }
        }
    }

    fn comment(&mut self, comment: &str) {
        self.commands
            .push(AsmIr::Comment(format!("// {}\n", comment)));
    }

    fn resolve_reference(&self, segment: &str) -> Result<String, String> {
        match segment {
            "local" => Ok("LCL".to_string()),
            "argument" => Ok("ARG".to_string()),
            "this" => Ok("THIS".to_string()),
            "that" => Ok("THAT".to_string()),
            _ => Err("Invalid segment".to_string()),
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn test_push() {
        let name = "test.vm".to_string();
        let mut translator = IrParser::new(&name);
        let command = VmCommand::Gt;
        let result = translator.parse(command);
        assert!(result.is_ok());

        let as_string = translator
            .commands
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("");

        println!("{}", as_string);
    }
}
