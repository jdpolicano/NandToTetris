use crate::alu::ALU;
use crate::instruction::Instruction;

const MAX_MEMORY: usize = 32_768; // 32KB the size of the hack computer memory.

#[derive(Debug, PartialEq)]
pub struct Cpu {
    pc: usize,
    d_reg: i16,
    a_reg: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0,
            d_reg: 0,
            a_reg: 0,
        }
    }

    pub fn run_next_instruction(&mut self, rom: &[u16], ram: &mut [i16]) {
        let instruction = self.fetch_instruction(rom);
        // print instruction as a debug message, so a 16 bit number is printed as binary
        if instruction.is_address() {
            self.a_reg = instruction.inner();
            return;
        }

        let alu = ALU::from_bits(instruction.comp_bits());
        let result = if instruction.a_bit_is_set() {
            #[cfg(debug_assertions)]
            self.debug_check_memory_bounds(self.a_reg as usize); // Only runs in debug mode
            alu.execute(self.d_reg, ram[self.a_reg as usize])
        } else {
            alu.execute(self.d_reg, self.a_reg as i16)
        };

        if instruction.dest_mem() {
            #[cfg(debug_assertions)]
            self.debug_check_memory_bounds(self.a_reg as usize); // Only runs in debug mode
            ram[self.a_reg as usize] = result;
        }

        if instruction.dest_addr() {
            self.a_reg = result as u16;
        }

        if instruction.dest_data() {
            self.d_reg = result;
        }

        if instruction.jump().cmp(result) {
            #[cfg(debug_assertions)]
            self.debug_check_rom_bounds(self.a_reg as usize); // Only runs in debug mode
            self.pc = self.a_reg as usize;
        }
    }

    fn fetch_instruction(&mut self, rom: &[u16]) -> Instruction {
        #[cfg(debug_assertions)]
        self.debug_check_rom_bounds(self.pc); // Only runs in debug mode
        let ret = Instruction::new(rom[self.pc]);
        self.pc += 1;
        ret
    }

    #[cfg(debug_assertions)]
    fn debug_check_memory_bounds(&self, addr: usize) {
        if addr >= MAX_MEMORY {
            panic!("Memory access out of bounds! Address: {}", addr);
        }
    }

    #[cfg(debug_assertions)]
    fn debug_check_rom_bounds(&self, addr: usize) {
        if addr >= MAX_MEMORY {
            panic!("PC out of bounds! Address: {}", addr);
        }
    }
}
