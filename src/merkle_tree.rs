use crate::io_utils::{read_doc, read_node, read_summary, write_node, write_summary};
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

    pub fn load() -> Result<Option<Self>> {
        // Read summary content
        let summary = match read_summary()? {
            None => return Ok(None),
            Some(summary) => summary,
        };
        // Parse the first line
        let first_line = summary.lines().next().unwrap_or_default();
        // Parse the number of documents
        let n_field = first_line.split(":").nth(4);
        // Cast to usize and return the tree
        match n_field.and_then(|n| n.parse::<usize>().ok()) {
            Some(n) => Ok(Some(Self::new(n))),
            None => Ok(None),
        }
    }

    pub fn build(&self) -> Result<()> {
        // Compute and store doc hashes
        for j in 0..self.n {
            self.compute_doc(j)?;
        }
        // Compute and store node hashes
        for i in 1..=self.max_layer {
            let mut j: usize = 0;
            loop {
                let Some(left) = read_node(i - 1, 2 * j)? else {
                    break; // No more nodes in this layer
                };
                // Read the right node, otherwise use the empty vector
                let right = read_node(i - 1, 2 * j + 1)?.unwrap_or_default();
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

    pub fn add_doc(&mut self, leaf_idx: usize) -> Result<()> {
        // Compute and store doc hash
        self.compute_doc(leaf_idx)?;
        // Increment number of documents
        self.n += 1;
        self.max_layer = max_layer(self.n);

        let mut j = leaf_idx / 2;
        // Update node hashes
        for i in 1..=self.max_layer {
            // Read the left node
            let left = read_node(i - 1, 2 * j)?.expect("Left node must exist");
            // Read the right node, otherwise use the empty vector
            let right = read_node(i - 1, 2 * j + 1)?.unwrap_or_default();
            // Compute hash of the parent node
            let parent = blake3(NOD_PREFIX.as_bytes(), &[&left, &right]);
            // Write node to file
            write_node(i, j, &parent)?;
            // Move up
            j /= 2;
        }
        Ok(())
    }

    pub fn store(&self) -> Result<()> {
        write_summary(self.summary()?)
    }

    // Private methods
    fn summary(&self) -> Result<Vec<String>> {
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

    fn compute_doc(&self, j: usize) -> Result<()> {
        // Read document content
        let data = read_doc(j)?;
        // Compute blake3 hash
        let digest = blake3(DOC_PREFIX.as_bytes(), &[&data]);
        // Write hash to file
        write_node(0, j, &digest)?;
        Ok(())
    }
}
