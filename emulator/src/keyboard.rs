use crate::ram::Ram;
use std::collections::HashMap;
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

enum KeyboardState {
    Pressed(Key),
    None,
}

pub struct Keyboard {
    state: KeyboardState,
    ram: Ram,
    keyboard_address: usize,
}

impl Keyboard {
    pub fn new(ram: Ram, keyboard_address: usize) -> Self {
        Self {
            state: KeyboardState::None,
            ram,
            keyboard_address,
        }
    }

    pub fn press(&mut self, key: Key) {
        self.state = KeyboardState::Pressed(key);
    }

    pub fn release(&mut self) {
        self.state = KeyboardState::None;
    }

    pub fn get_key(&self) -> Option<Key> {
        match &self.state {
            KeyboardState::Pressed(key) => Some(key.clone()),
            KeyboardState::None => None,
        }
    }

    pub fn get_key_string(&self) -> Option<String> {
        match &self.state {
            KeyboardState::Pressed(key) => key.to_text().map(|s| s.to_string()),
            KeyboardState::None => None,
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) {
        match self.state {
            KeyboardState::Pressed(_) => {
                self.handle_release(event);
            }
            KeyboardState::None => {
                self.handle_press(event);
            }
        }
    }

    fn handle_press(&mut self, event: KeyEvent) {
        match event.logical_key {
            Key::Named(NamedKey::ArrowLeft) => {
                self.press(Key::Named(NamedKey::ArrowLeft));
                self.ram.write(self.keyboard_address, 130);
            }

            Key::Named(NamedKey::ArrowRight) => {
                self.press(Key::Named(NamedKey::ArrowRight));
                self.ram.write(self.keyboard_address, 132);
            }

            _ => {}
        }
    }

    fn handle_release(&mut self, event: KeyEvent) {
        if let Some(key) = self.get_key() {
            if key != event.logical_key {
                return;
            }
        }
        self.release();
        self.ram.write(self.keyboard_address, 0);
    }
}

// /// Maps from the key event to the u16 value that the keyboard should write to memory.
// /// This is based on the hack specification.
// fn get_key_hashmap(key: Key) -> HashMap<Key, i16> {
//     let mut map = HashMap::with_capacity(130); // 130 is the number of keys we are mapping\
//     map.insert(Key::Named(NamedKey::Space), 32);
//     map.insert(Key::Character("!"), 33);
//     map
// }
