mod io_utils;
mod merkle_tree;

use crate::merkle_tree::MerkleTree;
use anyhow::Result;
use clap::{Parser, Subcommand};

static DOC_PREFIX: &str = "3C3C3C3C";
static NOD_PREFIX: &str = "F5F5F5F5";
static DATA_PATH: &str = "./data";

#[derive(Parser, Debug)]
#[command(name = "Merkle Tree", version, about = "Merkle Tree Generator")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build a new Merkle tree from N documents
    Build { n: usize },
    /// Add a new document to the tree
    Add { doc_idx: usize },
    /// Generate a proof for a given document
    Gen { doc_idx: usize },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Build { n } => {
            assert!(n > 0, "Number of documents must be greater than 0");
            // Create a new Merkle tree
            let tree = MerkleTree::new(n);
            // Build the tree
            tree.build()?;
            // Store the summary
            tree.store()?;
            println!("Tree computed successfully");
        }
        Command::Add { doc_idx } => {
            let mut tree = MerkleTree::load()?.expect("Tree not found");
            // Add the document to the tree
            tree.add_doc(doc_idx)?;
            // Store the summary
            tree.store()?;
            println!("New document added successfully");
        }
        Command::Gen { doc_idx } => {
            let tree = MerkleTree::load()?.expect("Tree not found");
            assert!(doc_idx < tree.elements(), "Invalid document index");
            // Generate the proof
            let proofs = tree.gen_proof(doc_idx)?;
            println!("Proofs generated successfully");
            for proof in proofs {
                println!("{}", proof);
            }
        }
    }
    Ok(())
}
