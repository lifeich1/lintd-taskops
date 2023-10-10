use anyhow::Context;
use anyhow::Result;
use duct::cmd;
use xtaskops::ops::clean_files;
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
        .run()
        .context("Failed cargo test")?;
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
    .run()?;
    clean_files("**/*.profraw")?;
    Ok(())
}
