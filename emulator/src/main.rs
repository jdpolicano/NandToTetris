use emulator::computer::{Computer, ComputerOptions};

// const MAX_MEMORY: usize = 32_768; // 32KB the size of the hack computer memory.

// #[derive(Debug, PartialEq)]
// struct Emulator {
//     ram: [i16; MAX_MEMORY],
//     rom: [u16; MAX_MEMORY],
//     pc: usize,
//     d_reg: i16,
//     a_reg: u16,
//     cycles: u128,
//     max_cycles: Option<u16>,
// }

// impl Emulator {
//     pub fn new(max_cycles: Option<u16>) -> Self {
//         Self {
//             ram: [0; MAX_MEMORY],
//             rom: [0; MAX_MEMORY],
//             pc: 0,
//             d_reg: 0,
//             a_reg: 0,
//             cycles: 0,
//             max_cycles,
//         }
//     }

//     pub fn load_rom(&mut self, rom: Vec<u16>) {
//         self.rom[..rom.len()].copy_from_slice(&rom);
//     }

//     pub fn run(&mut self) {
//         while self.should_run() {
//             let instruction = self.fetch_instruction();
//             // print instruction as a debug message, so a 16 bit number is printed as binary
//             if instruction.is_address() {
//                 self.a_reg = instruction.inner();
//                 self.cycles += 1;
//                 continue;
//             }

//             let alu = ALU::from_bits(instruction.comp_bits());
//             let result = if instruction.a_bit_is_set() {
//                 #[cfg(debug_assertions)]
//                 self.debug_check_memory_bounds(self.a_reg as usize); // Only runs in debug mode
//                 alu.execute(self.d_reg, self.ram[self.a_reg as usize])
//             } else {
//                 alu.execute(self.d_reg, self.a_reg as i16)
//             };

//             if instruction.dest_mem() {
//                 #[cfg(debug_assertions)]
//                 self.debug_check_memory_bounds(self.a_reg as usize); // Only runs in debug mode
//                 self.ram[self.a_reg as usize] = result;
//             }

//             if instruction.dest_addr() {
//                 self.a_reg = result as u16;
//             }

//             if instruction.dest_data() {
//                 self.d_reg = result;
//             }

//             if instruction.jump().cmp(result) {
//                 #[cfg(debug_assertions)]
//                 self.debug_check_rom_bounds(self.a_reg as usize); // Only runs in debug mode
//                 self.pc = self.a_reg as usize;
//             }

//             self.cycles += 1;
//         }
//     }

//     fn fetch_instruction(&mut self) -> Instruction {
//         #[cfg(debug_assertions)]
//         self.debug_check_rom_bounds(self.pc); // Only runs in debug mode
//         let ret = Instruction::new(self.rom[self.pc]);
//         self.pc += 1;
//         ret
//     }

//     #[cfg(debug_assertions)]
//     fn debug_check_memory_bounds(&self, addr: usize) {
//         if addr >= MAX_MEMORY {
//             panic!("Memory access out of bounds! Address: {}", addr);
//         }
//     }

//     #[cfg(debug_assertions)]
//     fn debug_check_rom_bounds(&self, addr: usize) {
//         if addr >= MAX_MEMORY {
//             panic!("PC out of bounds! Address: {}", addr);
//         }
//     }

//     fn should_run(&mut self) -> bool {
//         if let Some(ref mut max_cycles) = self.max_cycles {
//             if *max_cycles > 0 {
//                 *max_cycles -= 1;
//                 true
//             } else {
//                 false
//             }
//         } else {
//             true
//         }
//     }
// }

fn main() {
    let prog = read_prog("Prog.hack");
    let emulator = Computer::new(ComputerOptions {
        max_cycles: Some(1_000_000_000),
    });

    if emulator.is_err() {
        println!("Error: {:?}", emulator.err());
        return;
    }

    let mut emulator = emulator.unwrap();
    emulator.load_rom(prog);
    emulator.ram[0] = 250;
    let _ = emulator.run();
    // println!("RAM[0]: {:?}", emulator.ram[0]);
    // println!("RAM[256..265]: {:?}", &emulator.ram[256..266]);
}

fn read_prog(path: &str) -> Vec<u16> {
    let prog = std::fs::read_to_string(path).unwrap();
    read_prog_as_u16(&prog)
}

fn read_prog_as_u16(prog: &str) -> Vec<u16> {
    prog.lines()
        .map(|line| line.trim())
        .map(|line| u16::from_str_radix(line, 2).unwrap())
        .collect()
}
