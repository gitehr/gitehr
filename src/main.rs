use anyhow::Context;
use clap::{Parser, Subcommand};
use git2::{ObjectType, Oid, Repository};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use gitehr::commands;
use gitehr::error::Result;

#[derive(Parser)]
#[command(name = "gitehr")]
#[command(about = "A Git-based Electronic Health Record system", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init, // Initialize a new GitEHR repository (Wraps gitehr init)
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    }, // View details about a GitEHR object (Wraps git cat-file)
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    }, // Hash an object (Wraps git hash-object)
}

fn main() -> Result<()> {
    let args = Args::parse();

    eprintln!("--> [INFO]: Starting program");

    match args.command {
        Commands::Init => {
            commands::init::run()?;
        }
        Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            if !pretty_print {
                return Err(anyhow::anyhow!("parameter 'pretty_print' is required").into());
            }

            let repo = Repository::discover(".").context("locating Git repository")?;
            let oid = Oid::from_str(&object_hash).context("invalid object hash")?;
            let object = repo
                .find_object(oid, None)
                .with_context(|| format!("reading Git object {object_hash}"))?;

            match object.kind() {
                Some(ObjectType::Blob) => {
                    let blob = object.peel_to_blob().context("loading blob contents")?;
                    let stdout = io::stdout();
                    let mut stdout = stdout.lock();
                    stdout
                        .write_all(blob.content())
                        .context("writing blob to stdout")?;
                }
                Some(kind) => {
                    return Err(anyhow::anyhow!(format!(
                        "unsupported Git object type: {kind:?}"
                    ))
                    .into());
                }
                None => return Err(anyhow::anyhow!("unknown Git object type").into()),
            }
        }
        Commands::HashObject { write, file } => {
            let metadata = fs::metadata(&file)
                .with_context(|| format!("getting metadata for '{}'", file.display()))?;
            if !metadata.is_file() {
                return Err(anyhow::anyhow!("only regular files can be hashed").into());
            }

            if write {
                let repo = Repository::discover(".").context("locating Git repository")?;
                let oid = repo
                    .blob_path(&file)
                    .with_context(|| format!("writing blob for '{}'", file.display()))?;
                println!("{oid}");
            } else {
                let oid = Oid::hash_file(ObjectType::Blob, file.as_path())
                    .with_context(|| format!("hashing file '{}'", file.display()))?;
                println!("{oid}");
            }
        }
    }

    Ok(())
}
