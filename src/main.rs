mod relative_path;

use clap::Parser;
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Parser)]
struct Arguments {
    #[arg(short, long)]
    target: Option<PathBuf>,

    // todo: warn when non empty.
    children: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct ParsedArguments {
    target: PathBuf,
    children: Vec<PathBuf>,
}

impl TryFrom<Arguments> for ParsedArguments {
    type Error = std::io::Error;

    fn try_from(arguments: Arguments) -> Result<Self, Self::Error> {
        let target = arguments
            .target
            .map(|target| target.canonicalize())
            .unwrap_or_else(|| std::env::current_dir())?;

        Ok(Self {
            children: arguments.children,
            target,
        })
    }
}

fn find_checkable_directories(
    checkable_directories: &mut Vec<PathBuf>,
    target: &PathBuf,
    child: &PathBuf,
    from: &PathBuf,
) {
    let dir_entries = from
        .read_dir()
        .expect("[FATAL]: Expected to read directory entries from this dir");

    for dir_entry in dir_entries {
        let dir_entry = dir_entry
            .expect("[FATAL]: Expected to find a directory entry in the directory entries");

        let file_type = dir_entry
            .file_type()
            .expect("[FATAL]: Expected to get the file type from the directory entry");

        if !file_type.is_dir() {
            continue;
        }

        let to = dir_entry.path().splice(target, &child);

        let exists = to
            .try_exists()
            .expect("[FATAL]: Expected to check existence of path");

        if !exists {
            continue;
        }

        checkable_directories.push(to);

        find_checkable_directories(checkable_directories, target, child, from);
    }
}

fn main() {
    let args = Arguments::parse();

    let args = ParsedArguments::try_from(args)
        .expect("[FATAL]: Expected current working directory to exists");

    for child in args.children {
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
            println!("[WARN]: Skipping, expected the child to be a directory");
            continue;
        }

        // check directories first, after that we'll check for conflicts in individual files.

        let mut checkable_directories = Vec::new();

        find_checkable_directories(&mut checkable_directories, &args.target, &child, &from);

        // check each checkable directory if there's conflicts for files within.

        let mut conflicts = Vec::<PathBuf>::new();

        // iter non dirs only
        for dir in checkable_directories {
            for dir_entry in dir.read_dir().unwrap() {
                let dir_entry = dir_entry.unwrap();

                let file_type = dir_entry.file_type().unwrap();

                if file_type.is_dir() {
                    continue;
                }

                let to = dir_entry.path();

                if to.try_exists().unwrap() {
                    // could do this while we're iterating in fn above.
                    conflicts.push(to)
                }
            }
        }

        if conflicts.len() > 0 {
            println!("[WARNING]: Skipping directory, the following conflicts are present");

            for conflict in conflicts {
                println!("[WARNING]: {conflict:?}");
            }

            continue;
        }

        println!("from:     {from:?}");
        println!("target:   {:?}", args.target);

        // move the children

        for dir_entry in from.read_dir().unwrap() {
            let dir_entry = dir_entry.unwrap();

            println!("path: {:?}", dir_entry.path());
            println!("to:   {:?}", args.target);

            // we need to rename files, not directories.
            // so I'll get a list of file paths from, and to before hand and then
            // we can rename those here.
            fs::rename(dir_entry.path(), args.target.clone())
                .expect("[FATAL]: Expected to move the child into the parent");
        }

        fs::remove_dir_all(from).unwrap();
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
        let mids: usize = stop.as_ref().components().fold(0, |accu, _| accu + 1);
        let starts = start.as_ref().components();
        let long = self.components().skip(mids);

        let buffer = starts.chain(long).collect();

        buffer
    }
}
