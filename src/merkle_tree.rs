use crate::io_utils::{read_file, read_file_str, write_file, write_file_str};
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

#[inline]
fn max_layer(n: usize) -> usize {
    if n <= 1 {
        0
    } else {
        (usize::BITS - (n - 1).leading_zeros()) as usize
    }
}

#[inline]
fn is_even(n: usize) -> bool {
    n & 1 == 0
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

    pub fn elements(&self) -> usize {
        self.n
    }

    pub fn load() -> Result<Option<Self>> {
        // Read summary content
        let summary = read_file_str("summary.txt")?;
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
            // Traverse all the layer
            while let Ok(left) = read_file(format!("node{}.{}.dat", i - 1, 2 * j)) {
                // Read the right node, otherwise use the empty vector
                let right = read_file(format!("node{}.{}.dat", i - 1, 2 * j + 1))
                    .ok()
                    .unwrap_or_default();
                // Compute hash of the parent node
                let parent = blake3(NOD_PREFIX.as_bytes(), &[&left, &right]);
                // Write node to file
                write_file(format!("node{}.{}.dat", i, j), &parent)?;
                // Jump to the next node
                j += 1;
            }
        }
        Ok(())
    }

    pub fn add_doc(&mut self, doc_idx: usize) -> Result<()> {
        // Compute and store doc hash
        self.compute_doc(doc_idx)?;
        // Increment number of documents
        self.n += 1;
        self.max_layer = max_layer(self.n);

        let mut j = doc_idx / 2;
        // Update node hashes
        for i in 1..=self.max_layer {
            // Read the left node
            let left = read_file(format!("node{}.{}.dat", i - 1, 2 * j))?;
            // Read the right node, otherwise use the empty vector
            let right = read_file(format!("node{}.{}.dat", i - 1, 2 * j + 1))
                .ok()
                .unwrap_or_default();
            // Compute hash of the parent node
            let parent = blake3(NOD_PREFIX.as_bytes(), &[&left, &right]);
            // Write node to file
            write_file(format!("node{}.{}.dat", i, j), &parent)?;
            // Move up
            j /= 2;
        }
        Ok(())
    }

    pub fn gen_proof(&self, doc_idx: usize) -> Result<()> {
        let mut path: Vec<String> = Vec::with_capacity(self.max_layer);
        let mut j = doc_idx;
        // Generate proof
        for i in 0..self.max_layer {
            let entry = if is_even(j) {
                let sibling = read_file(format!("node{}.{}.dat", i, j + 1))
                    .ok()
                    .unwrap_or_default();
                format!("R{}", hex::encode(sibling))
            } else {
                let sibling = read_file(format!("node{}.{}.dat", i, j - 1))?;
                format!("L{}", hex::encode(sibling))
            };
            path.push(entry);
            // Move up
            j /= 2;
        }
        // Write proof to file
        write_file_str("proof.dat", path)?;
        Ok(())
    }

    pub fn verify_proof(pub_info: String, doc: String, proof: String) -> Result<bool> {
        // Extract required fields from public info
        let parts = pub_info.split(":").collect::<Vec<&str>>();
        let hasher = parts[1];
        assert_eq!(hasher, "blake3", "Only blake3 is supported for now");

        let doc_prefix = parts[2];
        let node_prefix = parts[3];
        let root_hex = parts[6];

        // Read and hash the document
        let doc = read_file_str(doc)?;
        let mut current = blake3(doc_prefix.as_bytes(), &[doc.as_bytes()]);

        // Process proof
        for entry in read_file_str(proof)?.lines() {
            let (dir, hash_hex) = entry.split_at(1);
            let hash = hex::decode(hash_hex)?;

            current = match dir {
                "L" => blake3(node_prefix.as_bytes(), &[&hash, &current]),
                "R" => blake3(node_prefix.as_bytes(), &[&current, &hash]),
                _ => anyhow::bail!("Invalid direction"),
            };
        }
        // Check if verification succeeds
        Ok(hex::encode(current) == root_hex)
    }

    pub fn store(&self) -> Result<()> {
        write_file_str("summary.txt", self.summary()?)
    }

    // Private methods
    fn summary(&self) -> Result<Vec<String>> {
        // Read root node
        let root = read_file(format!("node{}.{}.dat", self.max_layer, 0))?;

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
            // Traverse all the layer
            while let Ok(node) = read_file(format!("node{}.{}.dat", i, j)) {
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
        let data = read_file(format!("doc{}.dat", j))?;
        // Compute blake3 hash
        let digest = blake3(DOC_PREFIX.as_bytes(), &[&data]);
        // Write hash to file
        write_file(format!("node{}.{}.dat", 0, j), &digest)?;
        Ok(())
    }
}
