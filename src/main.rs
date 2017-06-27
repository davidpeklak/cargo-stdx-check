#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate docopt;
#[macro_use]
extern crate serde_derive;

extern crate toml;

use docopt::Docopt;
use std::env;
use std::fs::{copy, File, rename};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::Command;
use toml::value::{Value, Table};

const USAGE: &'static str = "
cargo-stdx-check

Usage:
    cargo-stdx-check (-h | --help)
    cargo-stdx-check test

Options:
    -h --help     Show this screen.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_test: bool
}

mod errors {
    error_chain! { }
}

use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let args: Result<Args> = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .chain_err(|| "bla");

    match args {
        Err(_) => {
            println!("{}", USAGE);
            Ok(())
        }
        Ok(ref ar) => if ar.cmd_test { test() } else { Ok(()) }
    }
}

fn test() -> Result<()> {
    let cargo = env::var("CARGO")
        .chain_err(|| "environment variable CARGO not set")?;

    let toml_str = read_string(Path::new("Cargo.toml"))?;

    let mut toml: Table = toml::from_str(&toml_str)
        .chain_err(|| "failed to parse Cargo.toml")?;

    {
        let maybe_deps = toml.get_mut("dependencies");
        if let Some(&mut Value::Table(ref mut deps)) = maybe_deps {
            deps.insert("stdx".to_string(), Value::String("0.117.0".to_string()));
        }
    }

    let toml_str = toml::to_string(&toml)
        .chain_err(|| "Cannot convert value to string")?;

    let _bkup_toml = Backup::new("Cargo.toml", "Cargo.toml.bk")?;

    write_string(Path::new("Cargo.toml"), &toml_str)
        .chain_err(|| "Failed to write Cargo.toml")?;

    let _bkup_lock = Backup::new("Cargo.lock", "Cargo.lock.bk")?;

    Command::new(cargo)
        .arg("test")
        .spawn()
        .expect("cargo test failed");

    Ok(())
}

fn read_string(path: &Path) -> Result<String> {
    let file = File::open(path)
        .chain_err(|| "Failed to open file")?;

    let mut f = BufReader::new(file);

    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .chain_err(|| "Failed to read to string from file")?;
    Ok(buf)
}

pub fn write_string(path: &Path, s: &str) -> Result<()> {
    let mut f = File::create(path)
        .chain_err(|| "Failed to create file")?;
    f.write_all(s.as_bytes())
        .chain_err(|| "Failed to write to file")?;
    Ok(())
}

struct Backup<'a> {
    orig: &'a str,
    bkup: &'a str
}

impl<'a> Backup<'a> {
    fn new(orig: &'a str, bkup: &'a str) -> Result<Backup<'a>> {
        rename(orig, bkup)
            .chain_err(|| "Failed to rename file")?;
        Ok(Backup { orig: orig, bkup: bkup })
    }
}

impl<'a> Drop for Backup<'a> {
    fn drop(&mut self) {
        println!("Renaming {} to {}", self.bkup, self.orig);
        copy(self.bkup, self.orig)
            .expect("Failed to restore file");
    }
}
