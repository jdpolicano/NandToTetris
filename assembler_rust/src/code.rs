use crate::parser::{Address, Instruction, Parser};
use std::collections::HashMap;

pub struct CodeGenerator<'a> {
    out: String,
    parser: Parser<'a>,
    instruction_count: u16,
    pub symbol_table: SymbolTable,
    pub translation_table: TranslationTable,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            out: String::new(),
            parser: Parser::new(&src),
            instruction_count: 0,
            symbol_table: SymbolTable::new(),
            translation_table: TranslationTable::new(),
        }
    }

    pub fn take_code(self) -> String {
        self.out
    }

    pub fn generate(&mut self) -> Result<(), String> {
        let _ = self.build_symbol_table()?;
        let _ = self.parser.reset();
        while let Some(instruction) = self.parser.next_instruction()? {
            match instruction {
                Instruction::AInstruction(addr) => self.translate_a_instruction(addr)?,
                Instruction::CInstruction { dest, comp, jump } => {
                    self.translate_c_instruction(dest, comp, jump)?
                }
                _ => {
                    continue;
                }
            };
        }
        Ok(())
    }

    pub fn build_symbol_table(&mut self) -> Result<(), String> {
        while let Some(instruction) = self.parser.next_instruction()? {
            match instruction {
                Instruction::Label(label) => {
                    self.symbol_table
                        .add_label(&label, self.instruction_count)?;
                }
                _ => {
                    self.instruction_count += 1;
                }
            };
        }
        Ok(())
    }

    fn translate_c_instruction(
        &mut self,
        dest: Option<String>,
        comp: String,
        jump: Option<String>,
    ) -> Result<(), String> {
        let t_dest = self.get_dest(dest)?;
        let t_comp = self.get_comp(comp)?;
        let t_jump = self.get_jump(jump)?;
        self.out
            .push_str(&format!("111{:013b}\n", (t_comp | t_dest | t_jump)));
        Ok(())
    }

    fn translate_a_instruction(&mut self, address: Address) -> Result<(), String> {
        let val = match address {
            Address::Symbol(symbol) => {
                if let Some(val) = self.symbol_table.get_symbol(&symbol) {
                    val
                } else {
                    self.symbol_table.add_variable(&symbol)
                }
            }
            Address::NumericConstant(val) => val,
        };
        self.out.push_str(&format!("{:016b}\n", val));
        Ok(())
    }

    fn get_dest(&self, dest: Option<String>) -> Result<u16, String> {
        match dest {
            Some(dest) => Ok(*self.translation_table.get_dest(&dest)?),
            None => Ok(0),
        }
    }

    fn get_comp(&self, comp: String) -> Result<u16, String> {
        Ok(*self.translation_table.get_comp(&comp)?)
    }

    fn get_jump(&self, jump: Option<String>) -> Result<u16, String> {
        match jump {
            Some(jump) => Ok(*self.translation_table.get_jump(&jump)?),
            None => Ok(0),
        }
    }
}
#[derive(Debug)]
pub struct SymbolTable {
    map: HashMap<String, u16>,
    variable_counter: u16,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: SymbolTable::get_init_table(),
            variable_counter: 16,
        }
    }

    fn get_init_table() -> HashMap<String, u16> {
        let mut map = HashMap::with_capacity((2 as usize).pow(16));
        // virtual registers
        map.insert("R0".to_string(), 0);
        map.insert("R1".to_string(), 1);
        map.insert("R2".to_string(), 2);
        map.insert("R3".to_string(), 3);
        map.insert("R4".to_string(), 4);
        map.insert("R5".to_string(), 5);
        map.insert("R6".to_string(), 6);
        map.insert("R7".to_string(), 7);
        map.insert("R8".to_string(), 8);
        map.insert("R9".to_string(), 9);
        map.insert("R10".to_string(), 10);
        map.insert("R11".to_string(), 11);
        map.insert("R12".to_string(), 12);
        map.insert("R13".to_string(), 13);
        map.insert("R14".to_string(), 14);
        map.insert("R15".to_string(), 15);
        // special bindings for assembly code
        map.insert("SP".to_string(), 0);
        map.insert("LCL".to_string(), 1);
        map.insert("ARG".to_string(), 2);
        map.insert("THIS".to_string(), 3);
        map.insert("THAT".to_string(), 4);
        // virtual memory mapped regions
        map.insert("SCREEN".to_string(), 16384);
        map.insert("KEYBOARD".to_string(), 24577);
        map
    }

    pub fn add_label(&mut self, label: &str, val: u16) -> Result<(), String> {
        if self.map.contains_key(label) {
            return Err(format!("attempt to add duplicate label: {}", label));
        }
        self.map.insert(label.to_string(), val);
        Ok(())
    }

    pub fn add_variable(&mut self, variable: &str) -> u16 {
        if self.map.contains_key(variable) {
            return *self.map.get(variable).unwrap();
        }
        let out = self.variable_counter;
        self.map.insert(variable.to_string(), self.variable_counter);
        self.variable_counter += 1;
        out
    }

    pub fn get_symbol(&self, symbol: &str) -> Option<u16> {
        self.map.get(symbol).map(|i| *i)
    }

    pub fn has(&self, symbol: &str) -> bool {
        self.map.contains_key(symbol)
    }
}

pub struct TranslationTable {
    comp_map: HashMap<String, u16>,
    dest_map: HashMap<String, u16>,
    jump_map: HashMap<String, u16>,
}

impl TranslationTable {
    pub fn new() -> Self {
        Self {
            comp_map: Self::init_comp_table(),
            dest_map: Self::init_dest_table(),
            jump_map: Self::init_jump_table(),
        }
    }

    fn init_comp_table() -> HashMap<String, u16> {
        let mut map = HashMap::new();
        map.insert("0".to_string(), 0b0101010 << 6);
        map.insert("1".to_string(), 0b0111111 << 6);
        map.insert("-1".to_string(), 0b0111010 << 6);
        map.insert("D".to_string(), 0b0001100 << 6);
        map.insert("A".to_string(), 0b0110000 << 6);
        map.insert("M".to_string(), 0b1110000 << 6);
        map.insert("!D".to_string(), 0b0001101 << 6);
        map.insert("!A".to_string(), 0b0110001 << 6);
        map.insert("!M".to_string(), 0b1110001 << 6);
        map.insert("-D".to_string(), 0b0001111 << 6);
        map.insert("-A".to_string(), 0b0110011 << 6);
        map.insert("-M".to_string(), 0b1110011 << 6);
        map.insert("D+1".to_string(), 0b0011111 << 6);
        map.insert("A+1".to_string(), 0b0110111 << 6);
        map.insert("M+1".to_string(), 0b1110111 << 6);
        map.insert("D-1".to_string(), 0b0001110 << 6);
        map.insert("A-1".to_string(), 0b0110010 << 6);
        map.insert("M-1".to_string(), 0b1110010 << 6);
        map.insert("D+A".to_string(), 0b0000010 << 6);
        map.insert("D+M".to_string(), 0b1000010 << 6);
        map.insert("D-A".to_string(), 0b0010011 << 6);
        map.insert("D-M".to_string(), 0b1010011 << 6);
        map.insert("A-D".to_string(), 0b0000111 << 6);
        map.insert("M-D".to_string(), 0b1000111 << 6);
        map.insert("D&A".to_string(), 0b0000000 << 6);
        map.insert("D&M".to_string(), 0b1000000 << 6);
        map.insert("D|A".to_string(), 0b0010101 << 6);
        map.insert("D|M".to_string(), 0b1010101 << 6);
        map
    }

    fn init_dest_table() -> HashMap<String, u16> {
        let mut map = HashMap::new();
        map.insert("M".to_string(), 0b001 << 3);
        map.insert("D".to_string(), 0b010 << 3);
        map.insert("DM".to_string(), 0b011 << 3);
        map.insert("A".to_string(), 0b100 << 3);
        map.insert("AM".to_string(), 0b101 << 3);
        map.insert("AD".to_string(), 0b110 << 3);
        map.insert("ADM".to_string(), 0b111 << 3);
        map
    }

    fn init_jump_table() -> HashMap<String, u16> {
        let mut map = HashMap::new();
        map.insert("JGT".to_string(), 0b001);
        map.insert("JEQ".to_string(), 0b010);
        map.insert("JGE".to_string(), 0b011);
        map.insert("JLT".to_string(), 0b100);
        map.insert("JNE".to_string(), 0b101);
        map.insert("JLE".to_string(), 0b110);
        map.insert("JMP".to_string(), 0b111);
        map
    }

    pub fn get_comp(&self, comp: &str) -> Result<&u16, String> {
        self.comp_map
            .get(comp)
            .ok_or(format!("invalid comp translation \"{}\"", comp))
    }

    pub fn get_dest(&self, dest: &str) -> Result<&u16, String> {
        self.dest_map
            .get(dest)
            .ok_or(format!("invalid dest translation \"{}\"", dest))
    }

    pub fn get_jump(&self, jump: &str) -> Result<&u16, String> {
        self.jump_map
            .get(jump)
            .ok_or(format!("invalid jump translation \"{}\"", jump))
    }
}
