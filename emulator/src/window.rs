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
        let buffer = [0; SCREEN_SIZE];
        let mut window = Window::new(
            "Hack Emulator",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            WindowOptions::default(),
        )
        .map_err(|e| e.to_string())?;

        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|e| e.to_string())?;

        Ok(Self {
            window,
            buffer: [0; SCREEN_SIZE],
        })
    }

    pub fn draw(&mut self, screen: &[i16]) -> Result<(), String> {
        let mut buffer_index = 0;
        for &word in screen {
            let mut bits = word as u16;
            for _ in 0..16 {
                let color = if bits & 1 != 0 { BLACK } else { WHITE };
                bits >>= 1;
                self.buffer[buffer_index] = color;
                buffer_index += 1;
            }
        }

        self.window
            .update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|e| e.to_string())
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }
}
