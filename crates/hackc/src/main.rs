use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, ValueEnum};
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
    Asm,
    Hack,
}

impl Format {
    fn extension(self) -> &'static str {
        match self {
            Self::Asm => "asm",
            Self::Hack => "hack",
        }
    }

    fn default_emit(self) -> Format {
        match self {
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
    #[error("unsupported compilation route: {from} -> {emit}; asm -> hack is currently supported")]
    UnsupportedRoute { from: Format, emit: Format },
    #[error("unable to read `{path}`: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("unable to compile `{path}`: {source}")]
    Assemble {
        path: PathBuf,
        source: hack_assembler::AssembleError,
    },
    #[error("unable to write `{path}`: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

fn infer_format(path: &Path) -> Option<Format> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
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

fn run(args: Args) -> Result<PathBuf, CliError> {
    let from = args
        .from
        .or_else(|| infer_format(&args.input))
        .ok_or_else(|| CliError::UnknownInputFormat(args.input.clone()))?;
    let emit = args.emit.unwrap_or_else(|| from.default_emit());

    if (from, emit) != (Format::Asm, Format::Hack) {
        return Err(CliError::UnsupportedRoute { from, emit });
    }

    let source = fs::read_to_string(&args.input).map_err(|source| CliError::Read {
        path: args.input.clone(),
        source,
    })?;
    let contents = hack_assembler::assemble(&source).map_err(|source| CliError::Assemble {
        path: args.input.clone(),
        source,
    })?;
    let output = output_path(&args.input, args.output, emit);
    fs::write(&output, contents).map_err(|source| CliError::Write {
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
