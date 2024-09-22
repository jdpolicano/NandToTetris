use crate::chipset::Chipset;
use crate::events::{CpuThreadMessage, MainThreadMessage};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
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
}

impl CpuThread {
    /// creates a new CpuThread with the given Chipset, Receiver and Sender
    pub fn new(
        chipset: Chipset,
        rx: Receiver<MainThreadMessage>,
        tx: Sender<CpuThreadMessage>,
        heartbeat_interval: Duration,
    ) -> Self {
        Self {
            chipset,
            rx,
            tx,
            state: State::Pending,
            last_hearbeat: Instant::now(),
            heartbeat_interval,
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
        let mut cycles = 0;
        while self.is_running() {
            self.check_messages();
            self.run_chipset();
            cycles += 1;
            if cycles % 1000 == 0 {
                std::thread::sleep(std::time::Duration::from_micros(250));
            }
        }
        println!("CPU thread finished {}", cycles);
    }

    /// runs the Chipset for a single cycle
    fn run_chipset(&mut self) {
        self.chipset.run_next_instruction();
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
