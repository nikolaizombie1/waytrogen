use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};
use std::env;
use std::io::Error;
use std::fs::create_dir_all;

const BIN_NAME: &str = "waytrogen";

include!("src/cli_parser.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
	None => return Ok(()),
	Some(outdir) => outdir,
    };
    let outdir = PathBuf::from(outdir).parent().unwrap().parent().unwrap().parent().unwrap().parent().unwrap().join("completions");
    create_dir_all(&outdir)?;
    let mut cli = Cli::command();
    for &shell in Shell::value_variants() {
	generate_to(shell, &mut cli, BIN_NAME, &outdir)?;
    }

    Ok(())
}
