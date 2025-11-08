mod io_utils;
mod merkle_tree;

use anyhow::{Result, ensure};
use clap::{Parser, Subcommand};
use merkle_tree::MerkleTree;

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
    Proof { doc_idx: usize },
    /// Verify a proof
    Verify { doc_name: String, proof: String },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Build { n } => {
            ensure!(n > 0, "Number of documents must be greater than 0");
            // Create a new Merkle tree
            let tree = MerkleTree::new(n);
            // Build the tree
            tree.build()?;
            // Store the summary
            tree.store()?;
            println!("Tree computed successfully");
        }
        Command::Add { doc_idx } => {
            let mut tree = MerkleTree::load()?;
            let n = tree.elements();
            ensure!(doc_idx == n, "Invalid document index, expected: {}", n);
            // Add the document to the tree
            tree.add_doc(doc_idx)?;
            // Store the summary
            tree.store()?;
            println!("New document added successfully");
        }
        Command::Proof { doc_idx } => {
            let tree = MerkleTree::load()?;
            let n = tree.elements();
            ensure!(doc_idx < n, "Choose a valid doc index in [0, {})", n);
            // Generate the proof
            tree.gen_proof(doc_idx)?;
            println!("Proof generated successfully");
        }
        Command::Verify { doc_name, proof } => {
            if MerkleTree::verify_proof(doc_name, proof)? {
                println!("Proof verification passed");
            } else {
                println!("Proof verification failed");
            }
        }
    }
    Ok(())
}
