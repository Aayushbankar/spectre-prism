use rs_merkle::{MerkleTree, Hasher};
use prism_common::Blake3Hash;
use anyhow::{Result, bail};

#[derive(Clone)]
pub struct Blake3Algorithm;

impl Hasher for Blake3Algorithm {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        blake3::hash(data).into()
    }
}

pub struct ProvenanceTree {
    leaves: Vec<[u8; 32]>,
}

impl Default for ProvenanceTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceTree {
    pub fn new() -> Self {
        Self { leaves: Vec::new() }
    }

    pub fn push_leaf(&mut self, hash: &Blake3Hash) -> Result<()> {
        if self.leaves.len() >= (1 << 16) {
            bail!("16-level limit reached");
        }
        self.leaves.push((*hash).into());
        Ok(())
    }

    pub fn root_hash(&self) -> Option<[u8; 32]> {
        if self.leaves.is_empty() {
            return None;
        }
        let tree = MerkleTree::<Blake3Algorithm>::from_leaves(&self.leaves);
        tree.root()
    }
    
    pub fn reset(&mut self) {
        self.leaves.clear();
    }
    
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }
}
