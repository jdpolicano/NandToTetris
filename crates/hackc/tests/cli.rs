use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn assembles_to_a_derived_sibling_path() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Prog.asm");
    fs::write(&input, "@2\nD=A\n").unwrap();

    cargo_bin_cmd!("hackc")
        .arg(&input)
        .assert()
        .success()
        .stdout(predicate::eq(format!(
            "wrote {}\n",
            directory.path().join("Prog.hack").display()
        )));

    assert_eq!(
        fs::read_to_string(directory.path().join("Prog.hack")).unwrap(),
        "0000000000000010\n1110110000010000\n"
    );
}

#[test]
fn supports_format_override_and_an_explicit_output() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("source.txt");
    let output = directory.path().join("machine-code");
    fs::write(&input, "@0\n").unwrap();

    cargo_bin_cmd!("hackc")
        .args(["--from", "asm", "--emit", "hack", "--output"])
        .arg(&output)
        .arg(&input)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(output.with_extension("hack")).unwrap(),
        "0000000000000000\n"
    );
}

#[test]
fn explains_unknown_inputs_and_unsupported_routes() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Prog.txt");
    fs::write(&input, "@0\n").unwrap();

    cargo_bin_cmd!("hackc")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("pass --from <format>"));

    cargo_bin_cmd!("hackc")
        .args(["--from", "asm", "--emit", "asm"])
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unsupported compilation route: asm -> asm",
        ));
}

#[test]
fn translates_vm_to_derived_assembly_with_static_file_name() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Simple.vm");
    fs::write(&input, "push static 3\npop temp 0\n").unwrap();

    cargo_bin_cmd!("hackc")
        .arg(&input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Simple.asm"));

    assert_eq!(
        fs::read_to_string(directory.path().join("Simple.asm")).unwrap(),
        "// push static 3\n@Simple.3\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n// pop temp 0\n@SP\nAM=M-1\nD=M\n@R5\nM=D\n"
    );
}

#[test]
fn translates_vm_directly_to_hack() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Math.vm");
    fs::write(&input, "push constant 7\nneg\n").unwrap();

    cargo_bin_cmd!("hackc")
        .args(["--emit", "hack"])
        .arg(&input)
        .assert()
        .success();

    let assembly = "@7\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@SP\nA=M-1\nM=-M\n";
    assert_eq!(
        fs::read_to_string(directory.path().join("Math.hack")).unwrap(),
        hack_assembler::assemble(assembly).unwrap()
    );
}

#[test]
fn reports_vm_translation_errors() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Bad.vm");
    fs::write(&input, "pop pointer 2\n").unwrap();

    cargo_bin_cmd!("hackc")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid pointer index `2`"));
}
