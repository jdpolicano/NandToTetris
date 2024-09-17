use vm_translator_rust::args::AssemblerArgs;
use vm_translator_rust::code::translate;

fn main() {
    let program_args = AssemblerArgs::parse();
    if let Ok(args) = program_args {
        if let Err(e) = translate(args.src) {
            println!("[err] {e}")
        }
    } else {
        println!("usage: assembler <source file>");
        println!("reminder that the source file must begin with a capital letter and have a .vm extension");
    }
}
