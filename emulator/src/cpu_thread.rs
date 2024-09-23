use crate::chipset::Chipset;
use crate::events::{CpuThreadMessage, MainThreadMessage};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// an enum to represent the current state of the cpu thread.
#[derive(Debug, PartialEq, Eq)]
enum State {
    Pending,  // the thread is waiting to start
    Running,  // the thread is currently running
    Finished, // the thread has finished running
    Error,    // the thread has encountered an error
}

/// represents a thread of execution on a CPU. It contains the Chipset that the thread
/// is running on, which handles the actual execution of instructions + RAM access etc.
/// The thread is responsible for control flow, checking for messages from the main, and
/// communicating any failures with the main thread.
#[derive(Debug)]
pub struct CpuThread {
    chipset: Chipset,
    rx: Receiver<MainThreadMessage>,
    tx: Sender<CpuThreadMessage>,
    state: State,
    last_hearbeat: Instant,
    heartbeat_interval: Duration,
    throttler: CpuThrottle,
    total_cycles: u128,
}

impl CpuThread {
    /// creates a new CpuThread with the given Chipset, Receiver and Sender
    pub fn new(
        chipset: Chipset,
        rx: Receiver<MainThreadMessage>,
        tx: Sender<CpuThreadMessage>,
        heartbeat_interval: Duration,
    ) -> Self {
        // Define the target cycle rate (e.g., 1,000,000 cycles/sec)
        let target_cycle_rate = 31_250_000.0; // 1 MHz
        let sample_rate = Duration::from_millis(5);
        Self {
            chipset,
            rx,
            tx,
            state: State::Pending,
            last_hearbeat: Instant::now(),
            heartbeat_interval,
            throttler: CpuThrottle::new(target_cycle_rate, sample_rate),
            total_cycles: 0,
        }
    }

    pub fn spawn(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            self.start();
        })
    }

    /// starts the CPU thread, running the Chipset until it finishes or an error occurs
    pub fn start(&mut self) {
        self.wait_for_start();
        if self.state == State::Running {
            println!("CPU thread started");
        }

        let start = Instant::now();
        while self.is_running() {
            self.check_messages();
            self.run_chipset();
            self.inc_cycles();
            self.throttler.throttle();
        }

        let cycles = self.get_cycles();
        println!(
            "CPU thread finished {} cycles in {} seconds avg: {:} cycles/sec",
            cycles,
            start.elapsed().as_secs_f64(),
            cycles as f64 / start.elapsed().as_secs_f64()
        );
    }

    /// runs the Chipset for a single cycle
    fn run_chipset(&mut self) {
        self.chipset.run_next_instruction();
    }

    /// increments the total number of cycles run by the CPUs
    fn inc_cycles(&mut self) {
        self.total_cycles += 1;
    }

    fn get_cycles(&self) -> u128 {
        self.total_cycles
    }

    /// waits for the main thread to send a start message
    /// before starting the CPU thread
    fn wait_for_start(&mut self) {
        let mut max_retry = 2500;
        while max_retry > 0 {
            match self.rx.try_recv() {
                Ok(MainThreadMessage::CpuStart) => {
                    self.state = State::Running;
                    self.last_hearbeat = Instant::now();
                    break;
                }
                Ok(MainThreadMessage::Error) => {
                    self.tx.send(CpuThreadMessage::Error).unwrap();
                    break;
                }
                _ => {}
            }
            thread::sleep(std::time::Duration::from_millis(1));
            max_retry -= 1;
        }
    }

    fn is_running(&self) -> bool {
        self.state == State::Running
    }

    fn check_messages(&mut self) {
        if self.last_hearbeat.elapsed() > self.heartbeat_interval {
            // to-do: lets make this something the main thread asks for. we can keep track of the last time it
            // asked for a heartbeat and if it's been too long, we can send an error message
            self.try_send_message(CpuThreadMessage::Heartbeat);
            self.last_hearbeat = Instant::now();
            match self.rx.try_recv() {
                Ok(MainThreadMessage::Error) => {
                    let _ = self.tx.send(CpuThreadMessage::Error);
                    self.state = State::Error;
                }
                Ok(MainThreadMessage::Finished) => {
                    let _ = self.tx.send(CpuThreadMessage::Finished);
                    self.state = State::Finished;
                }
                _ => {
                    // what should we do here?
                }
            }
        }
    }

    fn try_send_message(&mut self, msg: CpuThreadMessage) {
        match self.tx.send(msg) {
            Ok(_) => {}
            Err(_) => {
                self.state = State::Error;
            }
        }
    }
}

/// `CpuThrottle` is responsible for throttling the CPU emulator to run at a specified
/// cycles per second (CPS). It periodically checks the number of instructions executed
/// and introduces a sleep delay if the emulator is running too fast.
#[derive(Debug)]
pub struct CpuThrottle {
    /// Desired number of cycles per second.
    requested_cycles_per_sec: f64,
    /// Interval at which to poll the CPU for instruction counts.
    poll_interval: Duration,
    /// Timestamp of the last poll.
    last_poll_time: Instant,
    /// Instruction count at the last poll.
    instruction_count: u64,
}

impl CpuThrottle {
    /// Creates a new `CpuThrottle`.
    ///
    /// # Arguments
    ///
    /// * `requested_cycles_per_sec` - The desired number of cycles per second.
    /// * `poll_interval` - How often to poll the CPU for instruction counts.
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// let throttle = CpuThrottle::new(1_000_000.0, Duration::from_millis(100));
    /// ```
    pub fn new(requested_cycles_per_sec: f64, poll_interval: Duration) -> Self {
        Self {
            requested_cycles_per_sec,
            poll_interval,
            last_poll_time: Instant::now(),
            instruction_count: 0,
        }
    }

    /// Updates the throttle based on the current instruction count.
    ///
    /// This method should be called periodically (e.g., after executing a batch of instructions)
    /// with the current total instruction count from the CPU emulator.
    ///
    /// # Arguments
    ///
    /// * `current_instruction_count` - The total number of instructions executed by the CPU.
    ///
    /// # Example
    ///
    /// ```
    /// throttle.throttle(current_instr_count);
    /// ```
    pub fn throttle(&mut self) {
        let elapsed = Instant::now().duration_since(self.last_poll_time);
        if elapsed >= self.poll_interval {
            let elapsed_secs = elapsed.as_secs_f64();
            // Calculate the ideal time that should have elapsed for the executed instructions
            let ideal_time = self.instruction_count as f64 / self.requested_cycles_per_sec;
            if elapsed_secs < ideal_time {
                let sleep_time = ideal_time - elapsed_secs;
                let sleep_duration = Duration::from_secs_f64(sleep_time);
                sleep(sleep_duration);
            }
            // Update the last poll time and instruction count for the next interval
            self.last_poll_time = Instant::now();
            self.instruction_count = 0;
        } else {
            self.instruction_count += 1;
        }
    }
}
