use crate::asm::AsmTranslator;
use crate::ir::IrParser;
use crate::parser::VmParser;
use std::path::PathBuf;

pub fn translate(path: PathBuf) -> Result<(), String> {
    let source = read_file(&path)?;
    let file_name = get_file_name(&path)?;
    let mut parser = VmParser::new(&source);
    let mut ir_parser = IrParser::new(&file_name);
    while let Ok(command) = parser.next_command() {
        ir_parser.parse(command)?;
    }
    let output_path = output_path(&path);
    ir_parser.optimize();
    let assembly = ir_parser
        .commands
        .into_iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>()
        .join("");
    write_file(&output_path, assembly)?;
    Ok(())
}

fn output_path(original: &PathBuf) -> PathBuf {
    let mut path = original.clone();
    path.set_extension("asm");
    path
}

fn read_file(path: &PathBuf) -> Result<String, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{e}"))?;
    Ok(src)
}

fn write_file(path: &PathBuf, content: String) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| format!("{e}"))
}

fn get_file_name(path: &PathBuf) -> Result<String, String> {
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str().and_then(|s| Some(s.to_string())))
        .ok_or("invalid file name structure")?;
    Ok(file_name)
}
