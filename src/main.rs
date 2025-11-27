use anyhow::Result;
use clap::{Parser, Subcommand};
use std::ffi::CStr;
use std::fs;
use flate2::read::ZlibDecoder;
use std::io::{BufReader, prelude::*};
use std::io;
use anyhow::Context;

mod commands;

#[derive(Parser)]
#[command(name = "gitehr")]
#[command(about = "A Git-based Electronic Health Record system", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init, // Initialize a new GitEHR repository
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    }, // View details about a GitEHR object
    // Add {
    //     /// Content of the clinical entry
    //     #[arg(help = "The content to add to the journal")]
    //     content: String,
    // },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Init => {
            // commands::initialise()?;

            fs::create_dir(".gitehr").unwrap();
            fs::create_dir(".gitehr/objects").unwrap();
            fs::create_dir(".gitehr/refs").unwrap();
            fs::create_dir(".gitehr/logs").unwrap();
            fs::create_dir(".gitehr/index").unwrap();
            fs::create_dir(".gitehr/config").unwrap();
            fs::write(".gitehr/HEAD", "ref: refs/heads/main").unwrap();

            println!("Initialized GitEHR repository");

        }
        Commands::CatFile { pretty_print, object_hash } => {
            let f = fs::File::open(format!(
                ".gitehr/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..],
            )).context("open in .gitehr/objects")?;

            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);

            let mut buf = Vec::new();
            z.read_until(0, &mut buf)
                .context("reading header from gitehr object")?; // Technically this should be a CStr if successful

            let header = CStr::from_bytes_with_nul(&buf).context("gitehr object is invalid")?;
            let header = header.to_str().context("gitehr object is  UTF-8")?;

            let Some(size) = header.strip_prefix("blob ") else {
                anyhow::bail!("gitehr object is not a blob");
            };

            let size = size.parse::<usize>().context("invalid blob size")?;
            buf.clear();
            buf.resize(size, 0); // Allocated items in vector to zero. Performance hit
            z.read_exact(&mut buf[..]).context("file contents not matching expectation")?;

            let n = z.read(&mut [0]).context("validate EOF")?;
            anyhow::ensure!(n == 0, ".gitehr object still having trailing bytes");

            // Important to note the end result is a binary blob
    }}

    Ok(())
}
