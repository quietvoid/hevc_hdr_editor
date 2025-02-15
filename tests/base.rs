use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "hevc_hdr_editor [OPTIONS] --config <CONFIG> [input_pos]",
        ));
    Ok(())
}

#[test]
fn version() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg("--version").assert();

    assert.success().stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn edit() -> Result<()> {
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/regular.hevc");
    let edit_config = Path::new("assets/example_config.json");

    let output_file = temp.child("output.hevc");
    let expected_file = Path::new("assets/regular_example_cfg.hevc");

    let assert = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("--input")
        .arg(input_file)
        .arg("--config")
        .arg(edit_config)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_file));

    Ok(())
}

#[test]
fn edit_mkv() -> Result<()> {
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/regular.mkv");
    let edit_config = Path::new("assets/example_config.json");

    let output_file = temp.child("output.hevc");
    let expected_file = Path::new("assets/regular_example_cfg.hevc");

    let assert = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("--input")
        .arg(input_file)
        .arg("--config")
        .arg(edit_config)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_file));

    Ok(())
}

#[test]
fn edit_multimsg_sei() -> Result<()> {
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/multimsg-sei.hevc");
    let edit_config = Path::new("assets/example_config.json");

    let output_file = temp.child("output.hevc");
    let expected_file = Path::new("assets/multimsg-sei-example-cfg.hevc");

    let assert = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("--input")
        .arg(input_file)
        .arg("--config")
        .arg(edit_config)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_file));

    Ok(())
}
