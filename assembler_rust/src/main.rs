use assembler_rust::args::AssemblerArgs;
use assembler_rust::code::CodeGenerator;
use std::fs;

fn main() {
    let program_args = AssemblerArgs::parse();

    match program_args {
        Ok(args) => {
            if let Err(e) = translate(args) {
                println!("[err] {e}")
            }
        }
        Err(e) => {
            println!("[err] {}", e);
        }
    }
}

fn translate(args: AssemblerArgs) -> Result<(), String> {
    println!("[info] reading source {:?}...", args.src);
    let raw_file = fs::read_to_string(args.src).map_err(|e| format!("{e}"))?;
    let mut code = CodeGenerator::new(&raw_file);
    code.build_symbol_table()?;
    code.generate()?;
    fs::write("Prog.hack", code.take_code()).map_err(|e| format!("{e}"))?;
    println!("[info] done");
    Ok(())
}
