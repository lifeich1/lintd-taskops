use anyhow::{anyhow, bail, Context, Result};
use duct::cmd;
use serde_derive::Deserialize;
use std::{
    fs::File,
    io::{BufReader, Read},
};
use toml::Table;
use xtaskops::ops::clean_files;

pub trait Recipe {
    /// Like make recipe, drop all outputs.
    ///
    /// # Errors
    /// Return error if recipe failed.
    fn go(&self) -> Result<()>;

    /// Recipe helper, attach cmd debug repr at failing status.
    ///
    /// # Errors
    /// Return error if expression failed.
    fn eval(&self) -> Result<String>;
}

impl Recipe for duct::Expression {
    fn go(&self) -> Result<()> {
        let txt = format!("{self:?}");
        println!("-- {txt}");
        self.run()?;
        Ok(())
    }

    fn eval(&self) -> Result<String> {
        let txt = format!("{self:?}");
        self.read().with_context(|| format!("eval: {txt}"))
    }
}

/// # Errors
/// print short hint to stderr and bail.
pub fn neo_coverage() -> Result<()> {
    do_neo_coverage().map_err(|e| {
        eprintln!("{:#}", &e);
        e
    })
}

fn do_neo_coverage() -> Result<()> {
    cmd!("cargo", "test")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", "cargo-test-%p-%m.profraw")
        .stderr_to_stdout()
        .stdout_capture()
        .go()?;
    cmd!(
        "grcov",
        ".",
        "--binary-path",
        "./target/debug/deps",
        "-s",
        ".",
        "-t",
        "coveralls",
        "--branch",
        "--ignore-not-existing",
        "--ignore",
        "../*",
        "--ignore",
        "/*",
        "--ignore",
        "xtask/*",
        "--ignore",
        "*/src/tests/*",
        "--token",
        "NO_TOKEN",
    )
    .go()?;
    clean_files("**/*.profraw")?;
    Ok(())
}

/// # Errors
/// print short hint to stderr and bail.
pub fn bump_version(bump: &str) -> Result<()> {
    check_wd_clean()?;
    println!("=== bump version ===");
    cmd!("cargo", "set-version", "--exclude", "xtask", "--bump", bump).go()?;
    let pkg = default_members()?
        .pop()
        .ok_or_else(|| anyhow!("bad default-members"))?;
    println!("pkg: {}", &pkg);
    let ver = package_version(&pkg)?;
    if cmd!("nix", "eval", ".#packages")
        .stderr_to_stdout()
        .read()
        .is_ok()
    {
        cmd!("nix-update", "default", "--flake", "--version", &ver).go()?;
    }
    cmd!(
        "git",
        "commit",
        "--all",
        "--message",
        format!(":bookmark: bump to v{ver}")
    )
    .go()?;
    Ok(())
}

fn read_toml(path: &str) -> Result<Table> {
    let f = File::open(path).context("open file")?;
    let mut buf_rd = BufReader::new(f);
    let mut buf = String::new();
    buf_rd.read_to_string(&mut buf).context("read_to_string")?;
    buf.parse().context("try into Table")
}

#[derive(Deserialize)]
struct WorkspaceMeta {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    members: Vec<String>,
}

#[derive(Deserialize)]
struct PackageMeta {
    package: Package,
}
#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
}

fn package_version(package: &str) -> Result<String> {
    let meta: PackageMeta = read_toml(&format!("./{package}/Cargo.toml"))?
        .try_into()
        .context("try into PackageMeta")?;
    assert_eq!(meta.package.name, package);
    Ok(meta.package.version)
}

fn default_members() -> Result<Vec<String>> {
    let ws: WorkspaceMeta = read_toml("./Cargo.toml")?
        .try_into()
        .context("try into WorkspaceMeta")?;
    Ok(ws
        .workspace
        .members
        .into_iter()
        .filter(|x| x != "xtask")
        .collect())
}

fn check_wd_clean() -> Result<()> {
    let status = cmd!("git", "status", "-s").eval()?;
    if !status.is_empty() {
        bail!("{status}\nWorking directory dirty !!");
    }
    Ok(())
}

fn check_main_branch() -> Result<()> {
    let branch = cmd!("git", "branch", "--show-current").eval()?;
    if matches!(branch.as_ref(), "master" | "main") {
        Ok(())
    } else {
        bail!("branch `{branch}` is not main branch")
    }
}

fn push_verbose() -> Result<()> {
    cmd!("git", "push", "--verbose").go()?;
    Ok(())
}

/// # Errors
/// print short hint to stderr and bail.
pub fn publish() -> Result<()> {
    check_wd_clean()?;
    check_main_branch()?;
    push_verbose()?;
    let pkgs = &default_members()?;
    println!("=== dry run check all ===");
    for pkg in pkgs {
        println!("=== checking {pkg} ===");
        cmd!(
            "cargo",
            "publish",
            "--registry",
            "crates-io",
            "-p",
            pkg,
            "--dry-run"
        )
        .go()?;
    }
    println!("=== do publish ===");
    for pkg in pkgs {
        println!("=== publishing {pkg} ===");
        cmd!("cargo", "publish", "--registry", "crates-io", "-p", pkg).go()?;
    }
    println!("=== github release ===");
    let ver = package_version(&pkgs[0])?;
    cmd!(
        "gh",
        "release",
        "create",
        format!("v{ver}"),
        "--generate-notes"
    )
    .go()?;
    cmd!("git", "fetch", "--tags").go()?;
    println!("=== bump patch version ===");
    bump_version("patch")?;
    push_verbose()?;
    Ok(())
}
