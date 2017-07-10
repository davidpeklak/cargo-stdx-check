#![recursion_limit = "1024"]

extern crate docopt;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use docopt::Docopt;
use std::env;
use std::collections::BTreeMap;
use std::fs::{copy, File, rename};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::Command;
use toml::value::{Value, Array, Table};

const USAGE: &'static str = "
stdx-check

Usage:
    stdx-check (-h | --help)
    stdx-check [options]
    stdx-check [options] test
    stdx-check [options] dupes

Options:
    -h --help              Show this screen.
    --stdxgit REPOSITORY   Git repoistory of stdx [default: https://github.com/brson/stdx.git].
    --stdxversion VERSION  Version of stdx.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_test: bool,
    cmd_dupes: bool,
    flag_stdxgit: Option<String>,
    flag_stdxversion: Option<String>
}

enum StdxOpt {
    Git(String),
    Version(String)
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
        }
        Ok(ref ar) => {
            let stdx_opt = StdxOpt::from(ar);
            if ar.cmd_test && !ar.cmd_dupes {
                with_backups(|cargo| test(cargo), &stdx_opt)
            } else if ar.cmd_dupes && !ar.cmd_test {
                with_backups(|cargo| dupes(cargo), &stdx_opt)
            } else {
                with_backups(|cargo| {
                    test(cargo)?;
                    dupes(cargo)
                }, &stdx_opt)
            }
        }
    }
}

fn print_usage() -> () {
    println!("
    {}
    ", USAGE);
}

fn with_backups<F>(fun: F, stdx_opt: &StdxOpt) -> Result<()>
    where F: Fn(&str) -> Result<()> {
    let cargo = env::var("CARGO")
        .chain_err(|| "environment variable CARGO not set")?;

    let mut toml = parse_cargo_toml("Cargo.toml")?;

    let toml = insert_stdx_dep(&mut toml, &stdx_opt)?;

    let toml_str = toml::to_string(&toml)
        .chain_err(|| "Cannot convert toml to string")?;

    let _bkup_toml = Backup::new("Cargo.toml", "Cargo.toml.bk")?;

    write_string(Path::new("Cargo.toml"), &toml_str)?;

    write_string(Path::new("Cargo.toml.bk2"), &toml_str)?;

    let _bkup_lock = Backup::new("Cargo.lock", "Cargo.lock.bk")?;

    fun(&cargo)
}

impl StdxOpt {
    fn from(args: &Args) -> StdxOpt {
        match (&args.flag_stdxversion, &args.flag_stdxgit) {
            (&Some(ref version), _) => StdxOpt::Version(version.clone()),
            (&None, &Some(ref git)) => StdxOpt::Git(git.clone()),
            _ => unreachable!() // because --stdxgit is defaulted in docopt
        }
    }
}

fn insert_stdx_dep<'a, 'b>(toml: &'a mut Table, stdx_opt: &'b StdxOpt) -> Result<&'a mut Table> {
    {
        let deps = get_dependencies(toml)?;

        let stdx = match stdx_opt {
            &StdxOpt::Version(ref version) => Value::String(version.clone()),
            &StdxOpt::Git(ref git) => {
                let mut stdx_dep = Table::new();
                stdx_dep.insert("git".to_string(), Value::String(git.clone()));
                Value::Table(stdx_dep)
            }
        };

        let was = deps.insert("stdx".to_string(), stdx);
        was.map(|version| {
            return Err::<(), errors::Error>(format!("stdx {} is already a dependecy", version).into())
        });
    }

    Ok(toml)
}

fn get_dependencies(toml: &mut Table) -> Result<&mut Table> {
    let deps: &mut toml::Value = toml.get_mut("dependencies")
        .ok_or::<errors::Error>("cannot find dependencies in Cargo.toml".into())?;

    match deps {
        &mut Value::Table(ref mut deps) => Ok(deps),
        _ => bail!("'dependencies' is is not a table")
    }
}

fn get_packages(toml: &mut Table) -> Result<&mut Array> {
    let paks: &mut toml::Value = toml.get_mut("package")
        .ok_or::<errors::Error>("cannot find packages in Cargo.toml".into())?;

    match paks {
        &mut Value::Array(ref mut paks) => Ok(paks),
        _ => bail!("'package' is is not an array table")
    }
}

fn test(cargo: &str) -> Result<()> {
    exec_cargo(cargo, "test")
}

fn dupes(cargo: &str) -> Result<()> {
    exec_cargo(cargo, "build")?;

    let mut lock = parse_cargo_toml("cargo.lock")?;

    let paks = get_packages(&mut lock)?;

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();

    let pak_names = paks
        .iter()
        .filter_map(extract_name);

    for name in pak_names {
        *counts.entry(name).or_insert(0) += 1;
    }

    for (name, count) in counts
        .iter()
        .filter(|&(_, count)| *count > 1) {
        println!("{} appears {} times", name, count)
    }

    Ok(())
}

fn extract_name(val: &toml::Value) -> Option<&str> {
    match val {
        &Value::Table(ref entries) => {
            match entries.get("name") {
                Some(&Value::String(ref name)) => Some(name),
                _ => None
            }
        }
        _ => Option::None
    }
}

fn parse_cargo_toml(path: &str) -> Result<Table> {
    let toml_str = read_string(Path::new(path))?;

    toml::from_str(&toml_str)
        .chain_err(|| "failed to parse Cargo.toml")
}

fn exec_cargo(cargo: &str, arg: &str) -> Result<()> {
    Command::new(cargo)
        .arg(arg)
        .status()
        .chain_err(|| format!("Failed to execute 'cargo {}'", arg))
        .and_then(|exit_status| {
            if exit_status.success() {
                Ok(())
            } else {
                bail!(format!("'cargo {}' failed", arg));
            }
        })
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
