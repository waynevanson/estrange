mod contains_file_symlink_in_directory;
mod relative_path;

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use contains_file_symlink_in_directory::ContainsDirectory;
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

fn main() -> Result<(), io::Error> {
    let args = Arguments::parse();

    let args = ParsedArguments::try_from(args)
        .expect("[FATAL]: Expected current working directory to exists");

    env_logger::builder().filter_level(args.log_filter).init();

    let mut moveable_files = Vec::new();
    let mut conflicts_files = Vec::new();
    let mut deletable_directories = Vec::new();

    // todo: buffer so we can have dry run
    for child in args.sources {
        // Check that the child exists in the target.
        let child = if child.is_absolute() {
            child.clone()
        } else {
            args.target.join(&child)
        };

        let is_directory = child.metadata()?.is_dir();

        if !is_directory {
            // todo: I mean this could just mean `rename`.
            warn!("Skipping, expected the child to be a directory");
            continue;
        }

        let partitioned_files = partition_file_conflicts(&args.target, &child)?;

        moveable_files.extend(partitioned_files.0);
        conflicts_files.extend(partitioned_files.1);

        let deletable_directory = get_deletable_directory(&child, &args.target)?;
        deletable_directories.push(deletable_directory.to_owned());
    }

    if conflicts_files.len() > 0 {
        warn!("Skipping directory, the following conflicts are present");

        for conflict in conflicts_files {
            warn!("  {:?} -> {:?}", conflict.0, conflict.1);
        }
    }

    let mut creatable_directories = Vec::new();
    for (_, to_file) in &moveable_files {
        if let Some(parent) = to_file.parent() {
            creatable_directories.push(parent);
        }
    }

    if args.dry_run {
        for parent in creatable_directories {
            info!("create: {parent:?}");
        }

        for (from_file, to_file) in moveable_files {
            info!("move:    {from_file:?} -> {to_file:?}");
        }

        for deletable_directory in deletable_directories {
            info!("remove: {deletable_directory:?}");
        }
    } else {
        for parent in creatable_directories {
            fs::create_dir_all(parent)?;
        }

        for (from_file, to_file) in moveable_files {
            fs::rename(from_file, to_file)?;
        }

        for deletable_directory in deletable_directories {
            fs::remove_dir_all(deletable_directory)?;
        }
    }

    Ok(())
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

fn partition_file_conflicts(
    target: &Path,
    child: &Path,
) -> Result<(Vec<(PathBuf, PathBuf)>, Vec<(PathBuf, PathBuf)>), io::Error> {
    let mut moveables = Vec::new();
    let mut conflicts = Vec::new();

    for dir_entry in FilesUnfollowed::from(child.read_dir()?) {
        let from = dir_entry?.to_path_buf();
        let to = from.to_owned().splice(target, child);

        if to.exists() {
            conflicts.push((from, to));
        } else {
            moveables.push((from, to));
        }
    }

    Ok((moveables, conflicts))
}

fn get_deletable_directory<'a>(current: &'a Path, target: &Path) -> Result<&'a Path, io::Error> {
    let mut iterator = current.ancestors().peekable();

    iterator.next();

    let mut next = iterator.next();

    while let Some(current) = next {
        if let Some(parent) = current.parent() {
            if parent == target
                || parent.contains_file_symlink_in_directory()?
                || iterator.peek().is_none()
            {
                return Ok(current);
            }

            next = iterator.next();
        } else {
            break;
        }
    }

    Err(io::Error::from(io::ErrorKind::NotFound))
}
