use anyhow::Result;
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::ffi::CStr;
use std::fs;
use std::path::PathBuf;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{BufReader, prelude::*};
use std::io;
use anyhow::Context;
use std::io::Write;

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
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    }, // Hash an object
    // Add {
    //     /// Content of the clinical entry
    //     #[arg(help = "The content to add to the journal")]
    //     content: String,
    // },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectKind {
    Blob,
    // Tree,
    // Commit,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    eprintln!("--> [INFO] Starting program");

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

            eprintln!("Initialized GitEHR repository");

        }
        // Should be able to 'cat' blobs, tree and commits
        Commands::CatFile { pretty_print, object_hash } => {

            anyhow::ensure!(pretty_print, "parameter 'pretty_print' is required");

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

            // TODO: I wonder if an enum for kind would be better?
            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!(".gitehr/object did not start with a known type: '{header}'");
            };

            let kind = match kind {
                "blob" => ObjectKind::Blob,
                // "tree" => ObjectKind::Tree,
                // "commit" => ObjectKind::Commit,
                _ => anyhow::bail!(".gitehr/object did not start with an implemented type: '{header}'"),
            };

            let size = size.parse::<usize>().context("invalid blob size")?;

            // TODO: Need to think of error handling here
            // Will not error if limit is exceeded, but will return 0
            let mut z = z.take(size as u64);

            match kind {
                ObjectKind::Blob => {
                    let stdout = io::stdout();
                    let mut stdout = stdout.lock();
                    let n = std::io::copy(&mut z, &mut stdout).context("writing .gitehr/object blob to stdout")?;
                    anyhow::ensure!(n == size.try_into().unwrap(), "blob size mismatch");
                }
                _ => anyhow::bail!(".gitehr/object did not start with an implemented type: '{header}'"),

            }
        }
        Commands::HashObject { write, file } => {
            let stat = fs::metadata(&file).context("getting file metadata")?;

            let mut buf = Vec::new();
            let mut file = fs::File::open(&file).context("opening file")?;
            file.read_to_end(&mut buf).context("reading file")?;

            let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
            write!(e, "blob ")?;
            write!(e, "{}\0", stat.len())?;
            // e.write_all(&buf)?;


            let compressed = e.finish().context("finishing zlib compression")?;
            // Here you can write `compressed` to a file or stdout as needed.


        }
    }

    Ok(())
}


struct HashWriter<W> {
    writer: W,
    hasher: Sha256,
}

impl<W> Write for HashWriter<W>
where W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}