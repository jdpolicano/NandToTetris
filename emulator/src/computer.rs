use crate::cpu::Cpu;
use crate::window::Screen;

const MAX_MEMORY: usize = 32_768; // 32KB the size of the hack computer memory.
const SCREEN_LOC: usize = 16384;
const SCREEN_SIZE: usize = 8192;

pub struct ComputerOptions {
    pub max_cycles: Option<u16>,
}

pub struct Computer {
    pub ram: [i16; MAX_MEMORY],
    rom: [u16; MAX_MEMORY],
    cpu: Cpu,
    screen: Screen,
    cycles: u128,
    max_cycles: Option<u16>,
}

impl Computer {
    pub fn new(options: ComputerOptions) -> Result<Self, String> {
        Ok(Self {
            ram: [0; MAX_MEMORY],
            rom: [0; MAX_MEMORY],
            cpu: Cpu::new(),
            screen: Screen::new()?,
            cycles: 0,
            max_cycles: options.max_cycles,
        })
    }

    pub fn load_rom(&mut self, rom: Vec<u16>) {
        self.rom[..rom.len()].copy_from_slice(&rom);
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.should_run() {
            self.cpu.run_next_instruction(&self.rom, &mut self.ram);
            if self.screen.is_open() {
                self.screen
                    .draw(&self.ram[SCREEN_LOC..SCREEN_LOC + SCREEN_SIZE])?;
            } else {
                break;
            }
            self.cycles += 1;
        }

        println!("Cycles: {}", self.cycles);
        println!("{:?}", &self.ram[SCREEN_LOC..SCREEN_LOC + SCREEN_SIZE]);
        Ok(())
    }

    pub fn should_run(&mut self) -> bool {
        match self.max_cycles {
            Some(ref mut max) => {
                if *max == 0 {
                    false
                } else {
                    *max -= 1;
                    true
                }
            }
            None => true,
        }
    }
}
