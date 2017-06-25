extern crate toml;

use std::env;
use std::fs::{copy, File, rename};
use std::io::{BufReader, Read, Result, Write};
use std::path::Path;
use std::process::Command;
use toml::value::{Value, Table};

fn main() {
    let cargo = env::var("CARGO")
        .expect("environment variable CARGO not set");

    let toml_str = read_string(Path::new("Cargo.toml"))
        .expect("No Cargo.toml");
    let mut toml: Table = toml::from_str(&toml_str)
        .expect("failed to parse Cargo.toml");

    rename("Cargo.toml", "Cargo.toml.bk")
        .expect("Failed to rename Cargo.toml to Cargo.toml.bk");
    rename("Cargo.lock", "Cargo.lock.bk")
        .expect("Failed to rename Cargo.lock to Cargo.lock.bk");

    {
        let maybe_deps = toml.get_mut("dependencies");
        if let Some(&mut Value::Table(ref mut deps)) = maybe_deps {
            deps.insert("stdx".to_string(), Value::String("0.117.0".to_string()));
        }
    }

    let toml_str = toml::to_string(&toml)
        .expect("Cannot convert value to string");

    write_string(Path::new("Cargo.toml"), &toml_str)
        .expect("Failed to write Cargo.toml");

    Command::new(cargo)
        .arg("test")
        .spawn()
        .expect("cargo test failed");

    copy("Cargo.lock.bk", "Cargo.lock")
        .expect("Failed to rename Cargo.lock.bk to Cargo.lock");
    rename("Cargo.toml.bk", "Cargo.toml")
        .expect("Failed to rename Cargo.toml.bk to Cargo.toml");
}

fn read_string(path: &Path) -> Result<String> {
    let mut f = BufReader::new(File::open(path)
        .expect("Failed to open file"));
    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .expect("Feiled to read to string");
    Ok(buf)
}

pub fn write_string(path: &Path, s: &str) -> Result<()> {
    let mut f = File::create(path)
        .expect("Failed to create file");
    f.write_all(s.as_bytes())
        .expect("Failed to write to file");
    Ok(())
}
