use crate::cpu::Cpu;
use crate::window::Screen;
use std::time::Instant;

const MAX_MEMORY: usize = 32_768; // 32KB the size of the hack computer memory.
const SCREEN_LOC: usize = 16384;
const SCREEN_SIZE: usize = 8192;

pub struct ComputerOptions {
    pub max_cycles: Option<u32>,
}

pub struct Computer {
    pub ram: [i16; MAX_MEMORY],
    rom: [u16; MAX_MEMORY],
    cpu: Cpu,
    screen: Screen,
    cycles: u128,
    max_cycles: Option<u32>,
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
            if self.screen.is_open() {
                self.screen
                    .draw(&self.ram[SCREEN_LOC..SCREEN_LOC + SCREEN_SIZE])?;
            } else {
                break;
            }

            let now = Instant::now();
            // let frame_frate = ; // 100 frames per second
            while now.elapsed().as_millis() < 1 {
                self.cycles += 1;
                // Wait for the frame to finish
                self.cpu.run_next_instruction(&self.rom, &mut self.ram);
            }
        }

        Ok(())
    }

    pub fn should_run(&mut self) -> bool {
        match self.max_cycles {
            Some(ref max) => {
                if self.cycles >= *max as u128 {
                    false
                } else {
                    true
                }
            }
            None => true,
        }
    }
}
