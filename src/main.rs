#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

extern crate toml;

use std::env;
use std::fs::{copy, File, rename};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::Command;
use toml::value::{Value, Table};

mod errors {
    error_chain! { }
}

use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let cargo = env::var("CARGO")
        .chain_err(|| "environment variable CARGO not set")?;

    let toml_str = read_string(Path::new("Cargo.toml"))
        .chain_err(|| "No Cargo.toml")?;
    let mut toml: Table = toml::from_str(&toml_str)
        .chain_err(|| "failed to parse Cargo.toml")?;

    rename("Cargo.toml", "Cargo.toml.bk")
        .chain_err(|| "Failed to rename Cargo.toml to Cargo.toml.bk")?;
    rename("Cargo.lock", "Cargo.lock.bk")
        .chain_err(|| "Failed to rename Cargo.lock to Cargo.lock.bk")?;

    {
        let maybe_deps = toml.get_mut("dependencies");
        if let Some(&mut Value::Table(ref mut deps)) = maybe_deps {
            deps.insert("stdx".to_string(), Value::String("0.117.0".to_string()));
        }
    }

    let toml_str = toml::to_string(&toml)
        .chain_err(|| "Cannot convert value to string")?;

    write_string(Path::new("Cargo.toml"), &toml_str)
        .chain_err(|| "Failed to write Cargo.toml")?;

    Command::new(cargo)
        .arg("test")
        .spawn()
        .expect("cargo test failed");

    copy("Cargo.lock.bk", "Cargo.lock")
        .chain_err(|| "Failed to rename Cargo.lock.bk to Cargo.lock")?;
    rename("Cargo.toml.bk", "Cargo.toml")
        .chain_err(|| "Failed to rename Cargo.toml.bk to Cargo.toml")?;

    Ok(())
}

fn read_string(path: &Path) -> Result<String> {
    let file = File::open(path)
        .chain_err(|| "Failed to open file")?;

    let mut f = BufReader::new(file);

    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .chain_err(|| "Feiled to read to string")?;
    Ok(buf)
}

pub fn write_string(path: &Path, s: &str) -> Result<()> {
    let mut f = File::create(path)
        .chain_err(|| "Failed to create file")?;
    f.write_all(s.as_bytes())
        .chain_err(|| "Failed to write to file")?;
    Ok(())
}
