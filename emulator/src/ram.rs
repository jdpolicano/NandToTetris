use std::sync::{Arc, Mutex};
/// a struct that represents a ram. It contains the main memory of the computer and
/// exposes an api to read and write to it, supporting multi-threading through a
/// Mutex.
#[derive(Clone, Debug)]
pub struct Ram {
    memory: Arc<Mutex<Vec<i16>>>,
    size: usize,
}

impl Ram {
    /// creates a new Ram with the given memory
    pub fn new(size: usize) -> Self {
        Self {
            memory: Arc::new(Mutex::new(vec![0; size])),
            size,
        }
    }

    /// reads a value from the memory at the given address
    pub fn read(&self, address: usize) -> i16 {
        let memory = self.memory.lock().unwrap();
        memory[address]
    }

    /// writes a value to the memory at the given address
    pub fn write(&self, address: usize, value: i16) {
        let mut memory = self.memory.lock().unwrap();
        memory[address] = value;
    }

    /// read a slice of memory from the given address, copying the values into the slice
    /// provided. Returns the number of bytes read.
    pub fn copy_slice(
        &self,
        start_address: usize,
        slice: &mut [i16],
    ) -> Result<usize, &'static str> {
        // ensure this wont panic
        if start_address + slice.len() > self.size {
            return Err("out of bounds read access");
        }
        slice.copy_from_slice(
            &self.memory.lock().map_err(|_| "lock poisoned")?
                [start_address..start_address + slice.len()],
        );
        Ok(slice.len())
    }
}
