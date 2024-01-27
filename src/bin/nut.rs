#![allow(clippy::cast_ptr_alignment, clippy::module_inception)]

use clap::{Args, Parser, Subcommand};
use hexdump::{hexdump_iter, sanitize_byte};
use nut::{Bucket, BucketStats, DBBuilder, DB};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn validate_path(p: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(p);

    if path.exists() {
        Ok(path)
    } else {
        Err("Path not exists".to_string())
    }
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Dump {
            id,
            length,
            path_shim,
        } => dump(DumpOptions {
            path: path_shim.path,
            id,
            len: length,
        }),
        Commands::Info { check, path_shim } => info(InfoOptions {
            path: path_shim.path,
            check,
        }),
        Commands::Pages { ids, path_shim } => pages(PagesOptions {
            path: path_shim.path,
            ids,
        }),
        Commands::Tree { path_shim } => tree(TreeOptions {
            path: path_shim.path,
        }),
        Commands::Check { path_shim } => check(CheckOptions {
            path: path_shim.path,
        }),
    };

    if let Err(result) = result {
        eprintln!("Error: {}", result);
        std::process::exit(1);
    };
}

#[derive(Debug, Parser)]
#[command(about="Nut Database CLI", author, version, long_about =  format!(
"{}

homepage:
{}",
DESCRIPTION, HOMEPAGE,
))]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Dumps hex of the page
    Dump {
        /// id of the page
        #[arg(short, long)]
        id: usize,

        /// print no more than provided length
        #[arg(short, long)]
        length: Option<u64>,

        #[command(flatten)]
        path_shim: PathShim,
    },
    /// Prints database info
    Info {
        /// run db check
        #[arg(short, long)]
        check: bool,

        #[command(flatten)]
        path_shim: PathShim,
    },
    /// Prints a table of pages with their type (Meta, Leaf, Branch, Freelist)
    ///
    /// Leaf and branch pages will show a key count in the "items" column while the
    /// freelist will show the number of free pages in the "items" column.
    /// The "overflow" column shows the number of blocks that the page spills over
    /// into. Normally there is no overflow but large keys and values can cause
    ///a single page to take up multiple blocks.
    Pages {
        /// if provided, info will be given only on requested ids. takes multiple values
        #[arg(short, long = "id")]
        ids: Option<Vec<usize>>,

        #[command(flatten)]
        path_shim: PathShim,
    },
    /// Prints buckets tree
    Tree {
        #[command(flatten)]
        path_shim: PathShim,
    },
    /// Runs an exhaustive check to verify that all pages are accessible or are marked as freed
    Check {
        #[command(flatten)]
        path_shim: PathShim,
    },
}
#[derive(Debug, Args)]
struct PathShim {
    /// path to database
    #[arg(short, long, value_parser=validate_path)]
    path: PathBuf,
}

#[derive(Debug)]
struct DumpOptions {
    path: PathBuf,
    id: usize,
    len: Option<u64>,
}

fn dump(o: DumpOptions) -> Result<(), String> {
    let mut file = std::fs::File::open(&o.path).unwrap();
    let page_size = DB::get_meta(&mut file)?.page_size;
    let offset = u64::from(page_size) * o.id as u64;
    let meta = file
        .metadata()
        .map_err(|_| "Can't get file meta info")?
        .len();
    if offset > meta {
        return Err("ID out of bounds".to_string());
    };
    let mut overflowbuf = [0u8; 4];

    file.seek(SeekFrom::Start(offset + 12))
        .map_err(|_| "Can't seek file")?;
    file.read_exact(&mut overflowbuf)
        .map_err(|_| "Can't read file")?;
    let overflow = unsafe { *(&overflowbuf[0] as *const u8 as *const u32) };
    let mut take = (u64::from(overflow) + 1) * u64::from(page_size);
    if let Some(len) = o.len {
        take = u64::min(take, len);
    };

    file.seek(SeekFrom::Start(offset))
        .map_err(|_| "Can't seek file")?;
    let mut bound = Read::by_ref(&mut file).take(take);

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    let mut obuf = [0u8; 1024];
    let newline = [0x0a];
    let mut written = 0;
    while let Ok(size) = bound.read(&mut obuf[written..]) {
        if size == 0 {
            if written > 0 {
                for line in hexdump_iter(&obuf[..written]) {
                    stdout
                        .write_all(line.as_bytes())
                        .map_err(|_| "Can't print output")?;
                    stdout
                        .write_all(&newline)
                        .map_err(|_| "Can't print output")?;
                }
            }
            break;
        }
        written += size;
        if written == 1024 {
            for line in hexdump_iter(&obuf) {
                stdout
                    .write_all(line.as_bytes())
                    .map_err(|_| "Can't print output")?;
                stdout
                    .write_all(&newline)
                    .map_err(|_| "Can't print output")?;
            }
        }
    }
    stdout.write(&[0x0a]).map_err(|_| "Can't print output")?;
    Ok(())
}

struct CheckOptions {
    path: PathBuf,
}

fn check(o: CheckOptions) -> Result<(), String> {
    use ansi_term::Color::{Green, Red};

    let db = DBBuilder::new(o.path).read_only(true).build()?;
    let tx = db.begin_tx()?;
    let receiver = tx.check();
    let mut errlen = 0;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for err in receiver {
        writeln!(stdout, "    {}", err).map_err(|_| "Can't write output")?;
        errlen += 1;
    }
    if errlen != 0 {
        return Err(Red.paint("Check failed").to_string());
    }
    writeln!(stdout, "{}", Green.paint("Everything ok")).map_err(|_| "Can't write output")?;
    Ok(())
}

struct InfoOptions {
    path: PathBuf,
    check: bool,
}

fn info(o: InfoOptions) -> Result<(), String> {
    {
        let db = DBBuilder::new(&o.path).read_only(true).build()?;
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        writeln!(&mut stdout, "Page size: {}", db.page_size()).map_err(|_| "Can't write output")?;
        writeln!(&mut stdout, "Freelist id: {}", db.meta()?.freelist)
            .map_err(|_| "Can't write output")?;

        {
            let tx = db.begin_tx()?;
            let mut stats = BucketStats {
                ..Default::default()
            };
            let mut count = 0;
            let mut bucket_keys = vec![];
            tx.for_each(Box::new(|key, bucket| -> Result<(), String> {
                stats += bucket.unwrap().stats();
                bucket_keys.push(key.to_vec());
                count += 1;
                Ok(())
            }))?;
            writeln!(&mut stdout, "\nBucket keys ({}):", count)
                .map_err(|_| "Can't write output")?;
            for key in bucket_keys {
                let sanitized: String = key.clone().into_iter().map(sanitize_byte).collect();
                writeln!(&mut stdout, "    {:<20} {:02X?}", sanitized, key)
                    .map_err(|_| "Can't write output")?;
            }
            writeln!(&mut stdout, "\n{:#?}", stats).map_err(|_| "Can't write output")?;
        }

        if o.check {
            writeln!(&mut stdout, "\nCheck:").map_err(|_| "Can't write output")?;
        }
    }

    if o.check {
        check(CheckOptions { path: o.path })?;
    }

    Ok(())
}

struct PagesOptions {
    path: PathBuf,
    ids: Option<Vec<usize>>,
}

fn pages(o: PagesOptions) -> Result<(), String> {
    use ansi_term::Color::Green;
    let db = DBBuilder::new(&o.path).read_only(true).build()?;
    let tx = db.begin_tx()?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    writeln!(
        &mut stdout,
        "{}",
        Green.paint("ID     Type        Count   Overflow")
    )
    .map_err(|_| "Can't write output")?;
    writeln!(&mut stdout, "------ ----------- ------- --------")
        .map_err(|_| "Can't write output")?;

    let freed = tx.freed()?;

    if let Some(ids) = &o.ids {
        for id in ids {
            if freed.contains_key(&(*id as u64)) {
                writeln!(&mut stdout, "{:>6} free", id).map_err(|_| "Can't write output")?;
            } else {
                match tx.page_info(*id) {
                    Err(_) => writeln!(&mut stdout, "{:>6} error", id)
                        .map_err(|_| "Can't write output")?,
                    Ok(None) => {
                        writeln!(&mut stdout, "{:>6} none", id).map_err(|_| "Can't write output")?
                    }
                    Ok(Some(p)) => writeln!(
                        &mut stdout,
                        "{:>6} {:<11} {:>7} {:>8}",
                        p.id,
                        &format!("{:?}", p.ptype),
                        p.count,
                        p.overflow_count
                    )
                    .map_err(|_| "Can't write output")?,
                }
            }
        }
    } else {
        let mut id = 0;
        while let Some(p) = tx.page_info(id)? {
            if freed.contains_key(&(id as u64)) {
                writeln!(&mut stdout, "{:>6} free", id).map_err(|_| "Can't write output")?;
            } else {
                writeln!(
                    &mut stdout,
                    "{:>6} {:<11} {:>7} {:>8}",
                    p.id,
                    &format!("{:?}", p.ptype),
                    p.count,
                    p.overflow_count
                )
                .map_err(|_| "Can't write output")?;
            }
            id += 1;
        }
    }
    Ok(())
}

struct TreeOptions {
    path: PathBuf,
}

fn tree_writer(
    indent_level: usize,
    mut out: &mut std::io::StdoutLock<'_>,
    key: &[u8],
    bucket: Option<&Bucket>,
) -> Result<(), String> {
    if let Some(ubucket) = bucket {
        let sanitized: String = key.iter().copied().map(sanitize_byte).collect();
        writeln!(
            &mut out,
            "{:<40} {:02X?}",
            format!("{}{}", " ".repeat(indent_level * 2), sanitized),
            key
        )
        .map_err(|_| "Can't write output")?;

        let buckets = ubucket.buckets();

        for b_name in buckets {
            tree_writer(indent_level + 1, out, &b_name, ubucket.bucket(&b_name))?;
        }
    };

    Ok(())
}

fn tree(o: TreeOptions) -> Result<(), String> {
    let db = DBBuilder::new(o.path).read_only(true).build()?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    writeln!(&mut stdout, "{:<40} binary", "key").map_err(|_| "Can't write output")?;
    writeln!(&mut stdout, "{} {}", "=".repeat(40), "=".repeat(20))
        .map_err(|_| "Can't write output")?;

    {
        let tx = db.begin_tx()?;
        tx.for_each(Box::new(
            |key: &[u8], bucket: Option<&Bucket>| -> Result<(), String> {
                tree_writer(0, &mut stdout, key, bucket)?;

                Ok(())
            },
        ))?;
    }

    Ok(())
}
