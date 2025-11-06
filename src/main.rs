mod io_utils;
mod merkle_tree;

use crate::merkle_tree::MerkleTree;
use anyhow::Result;
use clap::Parser;

static DOC_PREFIX: &str = "3C3C3C3C";
static NOD_PREFIX: &str = "F5F5F5F5";
static DATA_PATH: &str = "./data";

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Number of expected documents
    n: usize,
}

fn main() -> Result<()> {
    println!("Welcome to the Merkle Tree generator");

    let args = Args::parse();
    assert!(args.n > 0, "Number of documents must be greater than 0");
    // Create a new Merkle tree
    let tree = MerkleTree::new(args.n);
    // Build the tree
    tree.build()?;
    // Store the summary
    io_utils::write_summary(tree.summary()?)?;

    println!("Tree computed successfully");
    Ok(())
}
