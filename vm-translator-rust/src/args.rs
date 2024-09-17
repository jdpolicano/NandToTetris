use std::{env::args, path::PathBuf};

pub struct AssemblerArgs {
    pub src: PathBuf,
}

impl AssemblerArgs {
    pub fn parse() -> Result<AssemblerArgs, String> {
        let mut args = args();
        args.next();
        let src_str = args.next().ok_or("missing source file")?;
        let src = AssemblerArgs::validate_src(src_str.to_string())?;
        Ok(AssemblerArgs { src })
    }

    /// args for the program is a single positional argument: the source file
    /// The source file has two main requirements:
    /// 1. It must begin with a capital letter.
    /// 2. Its extension must be .vm
    fn validate_src(src: String) -> Result<PathBuf, String> {
        let src = PathBuf::from(src);
        if src.extension().is_none() {
            return Err("source file must have an extension".to_string());
        }
        if src.extension().unwrap() != "vm" {
            return Err("source file must have a .vm extension".to_string());
        }
        if let Some(file_name) = src.file_name() {
            if let Some(file_name) = file_name.to_str() {
                if !file_name.chars().next().unwrap().is_uppercase() {
                    return Err("source file must begin with a capital letter".to_string());
                }
            }
        }
        Ok(src)
    }
}
