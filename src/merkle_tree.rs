use crate::io_utils::{read_doc, read_node, write_node};
use crate::{DOC_PREFIX, NOD_PREFIX};
use anyhow::Result;

fn blake3(prefix: &[u8], data: &[&[u8]]) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prefix);
    for part in data {
        hasher.update(part);
    }
    hasher.finalize().as_bytes().to_vec()
}

fn max_layer(n: usize) -> usize {
    if n <= 1 {
        0
    } else {
        (usize::BITS - (n - 1).leading_zeros()) as usize
    }
}

pub struct MerkleTree {
    n: usize,
    max_layer: usize,
}

impl MerkleTree {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            max_layer: max_layer(n),
        }
    }

    pub fn compute(&self) -> Result<()> {
        self.compute_leaves()?;
        self.compute_nodes()?;
        Ok(())
    }

    fn compute_leaves(&self) -> Result<()> {
        // Compute leaf hashes
        for j in 0..self.n {
            // Read document content
            let data = read_doc(j)?;
            // Compute blake3 hash
            let digest = blake3(DOC_PREFIX.as_bytes(), &[&data]);
            // Write hash to file
            write_node(0, j, &digest)?;
        }
        Ok(())
    }

    fn compute_nodes(&self) -> Result<()> {
        // Compute node hashes
        for i in 1..=self.max_layer {
            let mut j: usize = 0;
            loop {
                let left_idx = 2 * j;
                let right_idx = 2 * j + 1;

                let Some(left) = read_node(i - 1, left_idx)? else {
                    break; // No more nodes in this layer
                };

                // Read the right node, otherwise use the empty vector
                let right = read_node(i - 1, right_idx)?.unwrap_or_default();

                // Compute hash of the parent node
                let parent = blake3(NOD_PREFIX.as_bytes(), &[&left, &right]);

                // Write node to file
                write_node(i, j, &parent)?;

                // Jump to the next node
                j += 1;
            }
        }
        Ok(())
    }

    pub fn summary(&self) -> Result<Vec<String>> {
        // Read root node
        let root = read_node(self.max_layer, 0)?.expect("Root node not found");

        // Produce summary
        let mut lines: Vec<String> = Vec::new();
        let header = format!(
            "MerkleTree:blake3:{}:{}:{}:{}:{}",
            DOC_PREFIX.to_uppercase(),
            NOD_PREFIX.to_uppercase(),
            self.n,
            self.max_layer + 1,
            hex::encode(root)
        );
        lines.push(header);

        for i in 0..=self.max_layer {
            let mut j: usize = 0;
            loop {
                let Some(node) = read_node(i, j)? else {
                    break; // No more nodes in this layer
                };

                // Store entry in summary
                lines.push(format!("{}:{}:{}", i, j, hex::encode(node)));

                // Jump to the next node
                j += 1;
            }
        }
        Ok(lines)
    }
}
