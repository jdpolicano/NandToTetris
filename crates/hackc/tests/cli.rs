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
fn reports_assembly_parse_errors_with_unicode_aware_locations() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Bad.asm");
    fs::write(&input, "// αβ\r\nD=💥\n").unwrap();

    cargo_bin_cmd!("hackc")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "{}:2:3: unexpected token `💥`",
            input.display()
        )));
}

#[test]
fn reports_vm_parse_errors_with_unicode_aware_locations() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Bad.vm");
    fs::write(&input, "// αβ\r\npush 💥 0\n").unwrap();

    cargo_bin_cmd!("hackc")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "{}:2:6: unexpected token `💥`",
            input.display()
        )));
}

#[test]
fn reports_eof_parse_errors_at_the_end_of_the_source() {
    let directory = tempfile::tempdir().unwrap();
    let assembly = directory.path().join("End.asm");
    let vm = directory.path().join("End.vm");
    fs::write(&assembly, "@").unwrap();
    fs::write(&vm, "push").unwrap();

    cargo_bin_cmd!("hackc")
        .arg(&assembly)
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "{}:1:2: unexpected end of input",
            assembly.display()
        )));
    cargo_bin_cmd!("hackc")
        .arg(&vm)
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "{}:1:5: unexpected end of input",
            vm.display()
        )));
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

#[test]
fn rejects_an_output_that_is_the_input_without_modifying_it() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Prog.asm");
    let source = "@2\nD=A\n";
    fs::write(&input, source).unwrap();

    cargo_bin_cmd!("hackc")
        .args(["--emit", "hack", "--output"])
        .arg(&input)
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "input and output refer to the same file",
        ));

    assert_eq!(fs::read_to_string(&input).unwrap(), source);
}

#[cfg(unix)]
#[test]
fn rejects_symlink_and_hard_link_aliases_of_the_input() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Prog.asm");
    let symlink_output = directory.path().join("symlink.hack");
    let hard_link_output = directory.path().join("hard-link.hack");
    let source = "@2\nD=A\n";
    fs::write(&input, source).unwrap();
    symlink(&input, &symlink_output).unwrap();
    fs::hard_link(&input, &hard_link_output).unwrap();

    for output in [&symlink_output, &hard_link_output] {
        cargo_bin_cmd!("hackc")
            .args(["--emit", "hack", "--output"])
            .arg(output)
            .arg(&input)
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "input and output refer to the same file",
            ));
    }

    assert_eq!(fs::read_to_string(&input).unwrap(), source);
}

#[test]
fn write_failures_preserve_existing_output_and_do_not_create_parents() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Prog.asm");
    let output_directory = directory.path().join("destination.hack");
    let missing_output = directory.path().join("missing").join("Prog.hack");
    fs::write(&input, "@2\nD=A\n").unwrap();
    fs::create_dir(&output_directory).unwrap();

    cargo_bin_cmd!("hackc")
        .args(["--output"])
        .arg(&output_directory)
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unable to write"));
    assert!(output_directory.is_dir());

    cargo_bin_cmd!("hackc")
        .args(["--output"])
        .arg(&missing_output)
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unable to write"));
    assert!(!directory.path().join("missing").exists());
}

#[test]
fn successful_writes_replace_an_existing_output() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("Prog.asm");
    let output = directory.path().join("Prog.hack");
    fs::write(&input, "@2\nD=A\n").unwrap();
    fs::write(&output, "old contents\n").unwrap();

    cargo_bin_cmd!("hackc")
        .args(["--output"])
        .arg(&output)
        .arg(&input)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(&output).unwrap(),
        "0000000000000010\n1110110000010000\n"
    );
}

#[test]
fn writes_a_bare_relative_output_in_the_current_directory() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("Prog.asm"), "@0\n").unwrap();

    cargo_bin_cmd!("hackc")
        .current_dir(directory.path())
        .args(["--output", "result.hack", "Prog.asm"])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(directory.path().join("result.hack")).unwrap(),
        "0000000000000000\n"
    );
}
