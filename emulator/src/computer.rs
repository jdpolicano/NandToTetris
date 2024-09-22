use crate::chipset::Chipset;
use crate::cpu_thread::CpuThread;
use crate::events::{CpuThreadMessage, MainThreadMessage};
use crate::ram::Ram;
use crate::screen::{Dimension, HackScreenBuffer, Scaler};
use pixels::{Pixels, SurfaceTexture};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct ComputerConstants {
    screen_location: (usize, usize),
    keyboard_location: usize,
    ram_size: usize,
    rom_size: usize,
    screen_update_interval: Duration,
}

impl Default for ComputerConstants {
    fn default() -> Self {
        Self {
            screen_location: (16384, 24576),
            keyboard_location: 24576,
            ram_size: 32_768,
            rom_size: 32_768,
            screen_update_interval: Duration::from_millis(1),
        }
    }
}

pub struct ComputerOptions {
    pub max_cycles: Option<u32>,
    pub config: ComputerConstants,
    pub screen_dimensions: Dimension,
}

impl Default for ComputerOptions {
    fn default() -> Self {
        Self {
            max_cycles: None,
            config: ComputerConstants::default(),
            screen_dimensions: Dimension::default(),
        }
    }
}

pub struct Computer {
    ram: Ram,
    rom: Vec<u16>,
    max_cycles: Option<u32>,
    constants: ComputerConstants,
    screen_dimensions: Dimension,
    window: Option<Window>,
    rx: Option<Receiver<CpuThreadMessage>>,
    tx: Option<Sender<MainThreadMessage>>,
    cpu_thread: Option<JoinHandle<()>>,
    pixels: Option<Pixels>,
}

impl Computer {
    pub fn new(options: ComputerOptions) -> Self {
        Self {
            ram: Ram::new(options.config.ram_size),
            rom: vec![0; options.config.rom_size],
            max_cycles: options.max_cycles,
            constants: options.config,
            screen_dimensions: options.screen_dimensions,
            window: None,
            pixels: None,
            rx: None,
            tx: None,
            cpu_thread: None,
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u16>) {
        self.rom[..rom.len()].copy_from_slice(&rom);
    }

    fn init_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let attributes = WindowAttributes::default()
            .with_title("Hack Emulator")
            .with_inner_size(LogicalSize::new(
                self.screen_dimensions.logical_width() as f32,
                self.screen_dimensions.logical_height() as f32,
            ));
        let window = event_loop
            .create_window(attributes)
            .map_err(|e| e.to_string())?;
        let inner_size = window.inner_size();
        println!("Window size: {:?}", inner_size);
        let surface_texture = SurfaceTexture::new(inner_size.width, inner_size.height, &window);
        let pixels = Pixels::new(
            inner_size.width as u32,
            inner_size.height as u32,
            surface_texture,
        )
        .map_err(|e| e.to_string())?;
        self.window = Some(window);
        self.pixels = Some(pixels);
        Ok(())
    }

    fn init_cpu(&mut self) -> Result<(), String> {
        let ram = self.ram.clone();
        let rom = self.rom.clone();
        let (tx_cpu, rx_cpu) = std::sync::mpsc::channel();
        let (tx_main, rx_main) = std::sync::mpsc::channel();
        let chipset = Chipset::new(rom, ram);
        let cpu = CpuThread::new(chipset, rx_main, tx_cpu, Duration::from_millis(10));
        self.cpu_thread = Some(cpu.spawn());
        self.rx = Some(rx_cpu);
        self.tx = Some(tx_main);
        Ok(())
    }

    fn send_message(&self, message: MainThreadMessage) -> Result<(), String> {
        if let Some(tx) = &self.tx {
            return tx.send(message).map_err(|e| e.to_string());
        }
        Err("No message channel".to_string())
    }

    fn render_hack_screen(&mut self) -> Result<(), String> {
        let (start, end) = self.constants.screen_location;
        let screen_words = end - start;
        let mut screen_state_container = vec![0i16; screen_words];
        self.ram.copy_slice(start, &mut screen_state_container)?;
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| "could not take ref of window".to_string())?;
        let dpi = window.scale_factor();
        let scaler = Scaler::new(dpi, dpi, &self.screen_dimensions);
        let screen_buffer = HackScreenBuffer::new(&screen_state_container);
        let pixels = self
            .pixels
            .as_mut()
            .ok_or_else(|| "no pixels".to_string())?;
        let frame_buffer = pixels.frame_mut();
        scaler.scale(screen_buffer.into_iter(), frame_buffer)?;
        pixels.render().map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl ApplicationHandler for Computer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let init_win = self.init_window(event_loop);
            if let Err(e) = init_win {
                println!("Error initializing window: {}", e);
                event_loop.exit();
            }

            let init_cpu = self.init_cpu();
            if let Err(e) = init_cpu {
                println!("Error initializing CPU: {}", e);
                event_loop.exit();
            }

            let msg_sent = self.send_message(MainThreadMessage::CpuStart);
            if let Err(e) = msg_sent {
                println!("Error sending message to CPU: {}", e);
                event_loop.exit();
            }

            self.window.as_ref().unwrap().request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                let msg_result = self.send_message(MainThreadMessage::Finished);
                if let Err(e) = msg_result {
                    println!("Error sending message to CPU: {}", e);
                }

                if let Some(cpu_thread) = self.cpu_thread.take() {
                    cpu_thread.join().unwrap();
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.
                // println!("Redraw requested");
                if let Err(e) = self.render_hack_screen() {
                    println!("Error rendering pixels: {}", e);
                    let _ = self.send_message(MainThreadMessage::Error);
                    event_loop.exit();
                    return;
                }

                self.window.as_ref().unwrap().request_redraw();
            }

            _ => (),
        }
    }
}
