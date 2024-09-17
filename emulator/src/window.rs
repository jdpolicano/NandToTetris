use minifb::{Key, Window, WindowOptions};

const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
const BLACK: u32 = 0x000000;
const WHITE: u32 = 0xFFFFFF;

pub struct Screen {
    window: Window,
    buffer: [u32; SCREEN_SIZE],
}

impl Screen {
    pub fn new() -> Result<Self, String> {
        let window = Window::new(
            "Hack Emulator",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            WindowOptions::default(),
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            window,
            buffer: [0; SCREEN_SIZE],
        })
    }

    pub fn draw(&mut self, screen: &[i16]) -> Result<(), String> {
        let mut changed = false;
        for (i, pixel) in screen.iter().enumerate() {
            for j in 0..16 {
                let color = if *pixel & (1 << (15 - j)) != 0 {
                    BLACK
                } else {
                    WHITE
                };

                if i * 16 + j >= SCREEN_SIZE {
                    break;
                }

                if color == self.buffer[i * 16 + j] {
                    continue;
                }
                changed = true;
                self.buffer[i * 16 + j] = color;
            }
        }

        if !changed {
            return Ok(());
        }

        self.window
            .update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|e| e.to_string())
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }
}
