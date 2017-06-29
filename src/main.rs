#![recursion_limit = "1024"]

extern crate docopt;
#[macro_use]
extern crate error_chain;
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
stdx-check

Usage:
    stdx-check (-h | --help)
    stdx-check test
    stdx-check dupes

Options:
    -h --help     Show this screen.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_test: bool,
    cmd_dupes: bool
}

mod errors {
    error_chain! { }
}

use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let args: Result<Args> = Docopt::new(USAGE)
        .map(|d| d.argv(env::args().skip(1)))
        .and_then(|d| d.parse())
        .and_then(|v| v.deserialize())
        .chain_err(|| "Failed to deserialize docopt");

    match args {
        Err(_) => {
            print_usage();
            Ok(())
        },
        Ok(ref ar) => if ar.cmd_test && !ar.cmd_dupes {
            with_backups(|cargo| test(cargo))
        } else if ar.cmd_dupes && !ar.cmd_test {
            with_backups(|cargo| dupes(cargo))
        } else {
            with_backups(|cargo| {
                test(cargo)?;
                dupes(cargo)
            })
        }
    }
}

fn print_usage() -> () {
    println!("
    {}
    ", USAGE);
}

fn with_backups<F>(fun: F) -> Result<()>
    where F: Fn(&str) -> Result<()> {
    let cargo = env::var("CARGO")
        .chain_err(|| "environment variable CARGO not set")?;

    let toml_str = read_string(Path::new("Cargo.toml"))?;

    let mut toml: Table = toml::from_str(&toml_str)
        .chain_err(|| "failed to parse Cargo.toml")?;

    {
        let maybe_deps = toml.get_mut("dependencies");
        if let Some(&mut Value::Table(ref mut deps)) = maybe_deps {
            deps.insert("stdx".to_string(), Value::String("0.1".to_string()));
        }
    }

    let toml_str = toml::to_string(&toml)
        .chain_err(|| "Cannot convert toml to string")?;

    let _bkup_toml = Backup::new("Cargo.toml", "Cargo.toml.bk")?;

    write_string(Path::new("Cargo.toml"), &toml_str)?;

    let _bkup_lock = Backup::new("Cargo.lock", "Cargo.lock.bk")?;

    fun(&cargo)
}

fn test(cargo: &str) -> Result<()> {
    Command::new(cargo)
        .arg("test")
        .status()
        .chain_err(|| "Failed to execute 'cargo test'")
        .and_then(|exit_status| {
            if exit_status.success() {
                Ok(())
            } else {
                bail!("'cargo test' failed");
            }
        })
}

fn dupes(_cargo: &str) -> Result<()> {
    println!("Checking dupes");
    Ok(())
}

fn read_string(path: &Path) -> Result<String> {
    let file = File::open(path)
        .chain_err(|| format!("Failed to open file {}", path.to_str().unwrap_or("()")))?;

    let mut f = BufReader::new(file);

    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .chain_err(|| format!("Failed to read to string from file {}", path.to_str().unwrap_or("()")))?;
    Ok(buf)
}

pub fn write_string(path: &Path, s: &str) -> Result<()> {
    let mut f = File::create(path)
        .chain_err(|| format!("Failed to create file {}", path.to_str().unwrap_or("()")))?;
    f.write_all(s.as_bytes())
        .chain_err(|| format!("Failed to write to file{}", path.to_str().unwrap_or("()")))
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
        copy(self.bkup, self.orig)
            .expect("Failed to restore file");
    }
}
