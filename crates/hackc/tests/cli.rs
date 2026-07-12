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
        .stdout(predicate::str::contains("Prog.hack"));

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
