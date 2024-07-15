use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Default, Parser)]
struct Arguments {
    children: Vec<PathBuf>
}

fn main() {
    let args = Arguments::parse();
    println!("Hello, {:?}!", args);
}
