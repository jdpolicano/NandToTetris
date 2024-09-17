use std::env;
use std::path::PathBuf;

pub struct AssemblerArgs {
    pub src: PathBuf,
}

impl AssemblerArgs {
    pub fn parse() -> Result<Self, &'static str> {
        let mut args = env::args();
        // skip executable path
        args.next();

        // see if there was an argument passed in.
        if let Some(path) = args.next() {
            return Ok(Self {
                src: PathBuf::from(path),
            });
        }

        Err("usage: cargo run [path]")
    }
}
