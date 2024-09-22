use emulator::computer::{Computer, ComputerOptions};
use winit::{event_loop::ControlFlow, event_loop::EventLoop};

fn main() {
    let prog = read_prog("Prog.hack");
    let options = ComputerOptions::default();
    let mut computer = Computer::new(options);
    computer.load_rom(prog);
    let event_loop = EventLoop::new().unwrap();
    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);
    match event_loop.run_app(&mut computer) {
        Ok(_) => println!("Computer ran successfully"),
        Err(e) => eprintln!("Error running computer: {}", e),
    }
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
