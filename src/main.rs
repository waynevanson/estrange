mod relative_path;

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use itertools::Itertools;
use log::{info, warn, LevelFilter};
use std::{
    fs::{self, ReadDir},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Parser)]
struct Arguments {
    /// Applies reading operations, prints writing operations.
    #[arg(short, long, alias = "dry")]
    dry_run: bool,

    /// The sources that we want to move into the target directory.
    /// Should be one or more sources, although this isn't validated yet.
    // todo: warn when non empty.
    sources: Vec<PathBuf>,

    /// The directory we want to move all our sources into.
    /// Defaults to the current working directory.
    #[arg(short, long)]
    target: Option<PathBuf>,

    // https://github.com/clap-rs/clap-verbosity-flag/blob/master/src/lib.rs
    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
}

#[derive(Debug, Clone)]
struct ParsedArguments {
    target: PathBuf,
    sources: Vec<PathBuf>,
    dry_run: bool,
    log_filter: LevelFilter,
}

impl TryFrom<Arguments> for ParsedArguments {
    type Error = std::io::Error;

    fn try_from(arguments: Arguments) -> Result<Self, Self::Error> {
        let target = arguments
            .target
            .map(|target| target.canonicalize())
            .unwrap_or_else(|| std::env::current_dir())?;

        Ok(Self {
            dry_run: arguments.dry_run,
            sources: arguments.sources,
            target,
            log_filter: arguments.verbosity.log_level_filter(),
        })
    }
}

fn main() {
    let args = Arguments::parse();

    let args = ParsedArguments::try_from(args)
        .expect("[FATAL]: Expected current working directory to exists");

    env_logger::builder().filter_level(args.log_filter).init();

    // todo: buffer so we can have dry run
    for child in args.sources {
        // Check that the child exists in the target.
        let from = if child.is_absolute() {
            child.clone()
        } else {
            args.target.join(&child)
        };

        let is_directory = from
            .metadata()
            .expect("[FATAL]: Expected to read the metadata from the path")
            .is_dir();

        if !is_directory {
            // todo: I mean this could just mean `rename`.
            warn!("Skipping, expected the child to be a directory");
            continue;
        }

        let (moveables, conflicts): (Vec<_>, Vec<_>) =
            FilesUnfollowed::from(from.read_dir().unwrap())
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .into_iter()
                .map(|filepath| (filepath.clone(), filepath.splice(&args.target, &from)))
                // todo - try_partition or something from itertools?
                .partition(|(_, to)| !to.try_exists().unwrap());

        if conflicts.len() > 0 {
            warn!("Skipping directory, the following conflicts are present");

            for conflict in conflicts {
                warn!("  {:?}", conflict.1);
            }
        }

        for (from, to) in moveables {
            if let Some(parent) = to.parent() {
                if args.dry_run {
                    info!("create: {parent:?}");
                } else {
                    fs::create_dir_all(parent).unwrap();
                }
            }

            if args.dry_run {
                info!("move:    {from:?} -> {to:?}");
            } else {
                fs::rename(from, to).unwrap()
            }
        }

        let deletable = from
            .ancestors()
            .skip(1)
            .take_while(|current| current != &args.target)
            .filter_map(|current| Some((current, current.parent()?)))
            .map(|(current, parent)| {
                parent
                    .read_dir()
                    .and_then(|read_dir| {
                        read_dir
                            .map(|dir_entry| dir_entry.and_then(|dir_entry| dir_entry.file_type()))
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .map(|file_types| (current, file_types))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("Should have no issues here")
            .into_iter()
            .find_or_last(|(_, file_types)| {
                file_types
                    .into_iter()
                    .any(|file_type| file_type.is_file() || file_type.is_symlink())
            })
            .map(|a| a.0)
            .expect("Expect this not to be empty");

        if args.dry_run {
            info!("remove: {deletable:?}");
        } else {
            fs::remove_dir_all(deletable).unwrap();
        }
    }
}

pub trait SplicePath {
    // todo: this makes a lot of assumptions but should be good enough for us.
    fn splice<P, Q>(&self, start: P, stop: Q) -> Self
    where
        Self: Sized,
        P: AsRef<Path>,
        Q: AsRef<Path>;
}

impl SplicePath for PathBuf {
    fn splice<P, Q>(&self, start: P, stop: Q) -> Self
    where
        Self: Sized,
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let start = start.as_ref();
        let stop = stop.as_ref();

        let starts = start.components();

        let mids: usize = stop.components().fold(0, |accu, _| accu + 1);
        let long = self.components().skip(mids);

        let buffer = starts.chain(long).collect();

        buffer
    }
}

struct FilesUnfollowed {
    read_dirs: Vec<ReadDir>,
}

impl From<ReadDir> for FilesUnfollowed {
    fn from(read_dir: ReadDir) -> Self {
        Self {
            read_dirs: vec![read_dir],
        }
    }
}

impl Iterator for FilesUnfollowed {
    type Item = Result<PathBuf, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // No iterators left, we're done.
        let mut item_option = self.read_dirs.last_mut()?.next();

        while let Some(result) = item_option {
            match result {
                Err(error) => return Some(Err(error)),
                Ok(dir_entry) => match dir_entry.file_type() {
                    Err(error) => return Some(Err(error)),
                    Ok(file_type) => {
                        if file_type.is_file() || file_type.is_symlink() {
                            return Some(Ok(dir_entry.path()));
                        } else if file_type.is_dir() {
                            let result = dir_entry.path().read_dir();

                            match result {
                                Err(error) => return Some(Err(error)),
                                Ok(read_dir) => self.read_dirs.push(read_dir),
                            }
                        }
                    }
                },
            }

            // go through all the read_dir iterators until they're empty
            item_option = self.read_dirs.last_mut()?.next();
        }

        None
    }
}
