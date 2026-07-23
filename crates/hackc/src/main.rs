use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, ValueEnum};
use same_file::is_same_file;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(version, about = "Compile Hack platform source files")]
struct Args {
    /// Source file to compile.
    input: PathBuf,

    /// Override the input format normally inferred from the extension.
    #[arg(long, value_enum)]
    from: Option<Format>,

    /// Emit an intermediate or final representation.
    #[arg(long, value_enum)]
    emit: Option<Format>,

    /// Output path. Defaults to the input path with the emitted extension.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Vm,
    Asm,
    Hack,
}

impl Format {
    fn extension(self) -> &'static str {
        match self {
            Self::Vm => "vm",
            Self::Asm => "asm",
            Self::Hack => "hack",
        }
    }

    fn default_emit(self) -> Format {
        match self {
            Self::Vm => Self::Asm,
            Self::Asm => Self::Hack,
            Self::Hack => Self::Hack,
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.extension())
    }
}

#[derive(Debug, Error)]
enum CliError {
    #[error("cannot infer the input format from `{0}`; pass --from <format>")]
    UnknownInputFormat(PathBuf),
    #[error(
        "unsupported compilation route: {from} -> {emit}; vm -> asm, vm -> hack, and asm -> hack are currently supported"
    )]
    UnsupportedRoute { from: Format, emit: Format },
    #[error("unable to read `{path}`: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path}:{line}:{column}: {source}")]
    AssembleParse {
        path: PathBuf,
        line: usize,
        column: usize,
        source: hack_assembler::ParseError,
    },
    #[error("unable to compile `{path}`: {source}")]
    AssembleCodegen {
        path: PathBuf,
        source: hack_assembler::CodegenError,
    },
    #[error("{path}:{line}:{column}: {source}")]
    TranslateParse {
        path: PathBuf,
        line: usize,
        column: usize,
        source: hack_vm_translator::ParseError,
    },
    #[error("unable to translate `{path}`: {source}")]
    TranslateCodegen {
        path: PathBuf,
        source: hack_vm_translator::CodegenError,
    },
    #[error("unable to assemble translated VM `{path}`: {source}")]
    AssembleInstructions {
        path: PathBuf,
        source: hack_assembler::CodegenError,
    },
    #[error("VM input path `{0}` does not have a valid UTF-8 file stem")]
    InvalidVmFileName(PathBuf),
    #[error("input and output refer to the same file: `{0}`")]
    SameInputAndOutput(PathBuf),
    #[error("unable to write `{path}`: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

fn infer_format(path: &Path) -> Option<Format> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "vm" => Some(Format::Vm),
        "asm" => Some(Format::Asm),
        "hack" => Some(Format::Hack),
        _ => None,
    }
}

fn output_path(input: &Path, requested: Option<PathBuf>, emit: Format) -> PathBuf {
    let is_derived = requested.is_none();
    let mut path = requested.unwrap_or_else(|| input.to_path_buf());
    if is_derived || path.extension().is_none() {
        path.set_extension(emit.extension());
    }
    path
}

fn paths_refer_to_same_file(input: &Path, output: &Path) -> std::io::Result<bool> {
    match is_same_file(input, output) {
        Ok(is_same) => Ok(is_same),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn write_atomically(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    temporary.write_all(contents.as_ref())?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

fn run(args: Args) -> Result<PathBuf, CliError> {
    let from = args
        .from
        .or_else(|| infer_format(&args.input))
        .ok_or_else(|| CliError::UnknownInputFormat(args.input.clone()))?;
    let emit = args.emit.unwrap_or_else(|| from.default_emit());

    if !matches!(
        (from, emit),
        (Format::Vm, Format::Asm | Format::Hack) | (Format::Asm, Format::Hack)
    ) {
        return Err(CliError::UnsupportedRoute { from, emit });
    }

    let output = output_path(&args.input, args.output, emit);
    if paths_refer_to_same_file(&args.input, &output).map_err(|source| CliError::Write {
        path: output.clone(),
        source,
    })? {
        return Err(CliError::SameInputAndOutput(output));
    }

    let source = fs::read_to_string(&args.input).map_err(|source| CliError::Read {
        path: args.input.clone(),
        source,
    })?;
    let contents = match (from, emit) {
        (Format::Asm, Format::Hack) => {
            hack_assembler::assemble(&source).map_err(|source| match source {
                hack_assembler::AssembleError::Parse(source) => CliError::AssembleParse {
                    path: args.input.clone(),
                    line: source.span.start.line,
                    column: source.span.start.column,
                    source,
                },
                hack_assembler::AssembleError::Codegen(source) => CliError::AssembleCodegen {
                    path: args.input.clone(),
                    source,
                },
            })?
        }
        (Format::Vm, Format::Asm | Format::Hack) => {
            let file_name = args
                .input
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| CliError::InvalidVmFileName(args.input.clone()))?;
            let instructions = hack_vm_translator::translate(&source, file_name).map_err(
                |source| match source {
                    hack_vm_translator::TranslateError::Parse(source) => CliError::TranslateParse {
                        path: args.input.clone(),
                        line: source.span.start.line,
                        column: source.span.start.column,
                        source,
                    },
                    hack_vm_translator::TranslateError::Codegen(source) => {
                        CliError::TranslateCodegen {
                            path: args.input.clone(),
                            source,
                        }
                    }
                },
            )?;

            if emit == Format::Asm {
                hack_assembler::format_instructions(&instructions)
            } else {
                hack_assembler::assemble_instructions(&instructions).map_err(|source| {
                    CliError::AssembleInstructions {
                        path: args.input.clone(),
                        source,
                    }
                })?
            }
        }
        _ => unreachable!("supported routes were checked above"),
    };
    write_atomically(&output, contents).map_err(|source| CliError::Write {
        path: output.clone(),
        source,
    })?;
    Ok(output)
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(output) => {
            println!("wrote {}", output.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_formats_case_insensitively() {
        assert_eq!(infer_format(Path::new("Prog.ASM")), Some(Format::Asm));
        assert_eq!(infer_format(Path::new("Prog.VM")), Some(Format::Vm));
    }

    #[test]
    fn adds_an_extension_to_an_explicit_extensionless_output() {
        assert_eq!(
            output_path(
                Path::new("Prog.asm"),
                Some(PathBuf::from("build/Prog")),
                Format::Hack
            ),
            PathBuf::from("build/Prog.hack")
        );
    }
}
