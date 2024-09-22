use crate::alu::Alu;
use crate::instruction::Instruction;
use crate::ram::Ram;

const MAX_MEMORY: usize = 32_768; // 32KB the size of the hack computer memory.

/// The Chipset struct represents the hardware of the Hack computer.
/// It contains the Arithmetic Logic Unit (ALU), the Program Counter (PC),
/// A register, and D register.
#[derive(Debug)]
pub struct Chipset {
    rom: Vec<u16>,
    ram: Ram,
    alu: Alu,
    pc: usize,
    d_reg: i16,
    a_reg: u16,
}

impl Chipset {
    pub fn new(rom: Vec<u16>, ram: Ram) -> Self {
        Self {
            rom,
            ram,
            alu: Alu::default(),
            pc: 0,
            d_reg: 0,
            a_reg: 0,
        }
    }

    pub fn run_next_instruction(&mut self) {
        let instruction = self.fetch_instruction();
        // print instruction as a debug message, so a 16 bit number is printed as binary
        if instruction.is_address() {
            self.a_reg = instruction.inner();
            return;
        }

        self.alu.load_bits(instruction.comp_bits());

        let result = if instruction.a_bit_is_set() {
            #[cfg(debug_assertions)]
            self.debug_check_memory_bounds(self.a_reg as usize);
            self.alu
                .execute(self.d_reg, self.ram.read(self.a_reg as usize))
        } else {
            self.alu.execute(self.d_reg, self.a_reg as i16)
        };

        if instruction.dest_mem() {
            #[cfg(debug_assertions)]
            self.debug_check_memory_bounds(self.a_reg as usize); // Only runs in debug mode
            self.ram.write(self.a_reg as usize, result);
        }

        if instruction.dest_addr() {
            #[cfg(debug_assertions)]
            self.debug_check_address_type(result); // Only runs in debug mode
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

    fn fetch_instruction(&mut self) -> Instruction {
        #[cfg(debug_assertions)]
        self.debug_check_rom_bounds(self.pc); // Only runs in debug mode
        let ret = Instruction::new(self.rom[self.pc]);
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

    #[cfg(debug_assertions)]
    fn debug_check_address_type(&self, addr: i16) {
        if addr < 0 {
            panic!(
                "Attempt to write negative value as unsigned integer {}",
                addr
            );
        }
    }
}
