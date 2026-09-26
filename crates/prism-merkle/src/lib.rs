//! PRISM Merkle Tree with 2-of-3 Ed25519 Witness Cosigning (RFC 6962)
//!
//! Based on ULPF's RFC 6962 implementation with added 2-of-3 Witness Cosigning.
//!
//! The construction follows RFC 6962 (Certificate Transparency), including its
//! domain separation: leaves are hashed with a `0x00` prefix and interior nodes
//! with `0x01`, so no interior node can ever be mistaken for a leaf.

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Verifier, Signer};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

pub type Hash = [u8; 32];

const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

/// Ed25519 key pair for witness signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessKeyPair {
    pub signing_key: Vec<u8>,   // 32 bytes
    pub verifying_key: Vec<u8>, // 32 bytes
}

impl WitnessKeyPair {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key: signing_key.to_bytes().to_vec(),
            verifying_key: verifying_key.to_bytes().to_vec(),
        }
    }

    pub fn from_bytes(signing_key: &[u8], verifying_key: &[u8]) -> Result<Self, anyhow::Error> {
        if signing_key.len() != 32 || verifying_key.len() != 32 {
            return Err(anyhow::anyhow!("Invalid key length"));
        }
        Ok(Self {
            signing_key: signing_key.to_vec(),
            verifying_key: verifying_key.to_vec(),
        })
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signing_key = SigningKey::from_bytes(self.signing_key.as_slice().try_into().unwrap());
        let signature = signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        let verifying_key = VerifyingKey::from_bytes(self.verifying_key.as_slice().try_into().unwrap()).unwrap();
        let signature = Signature::from_bytes(signature.try_into().unwrap());
        verifying_key.verify(message, &signature).is_ok()
    }
}

/// A witness signature on a Merkle root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessSignature {
    pub witness_index: usize,      // 0, 1, or 2
    pub root_hash: Vec<u8>,        // 32 bytes
    pub signature: Vec<u8>,        // 64 bytes
    pub timestamp: u64,            // Unix timestamp
}

impl WitnessSignature {
    pub fn new(witness_index: usize, root_hash: &[u8], signing_key: &SigningKey) -> Self {
        let signature = signing_key.sign(root_hash);
        Self {
            witness_index,
            root_hash: root_hash.to_vec(),
            signature: signature.to_bytes().to_vec(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn verify(&self, verifying_keys: &[VerifyingKey]) -> bool {
        if self.witness_index >= verifying_keys.len() {
            return false;
        }
        let sig = Signature::from_slice(&self.signature).ok();
        if sig.is_none() {
            return false;
        }
        verifying_keys[self.witness_index].verify(&self.root_hash, &sig.unwrap()).is_ok()
    }
}

/// A Merkle log with 2-of-3 Witness Cosigning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessMerkleLog {
    /// The underlying Merkle log
    merkle: MerkleLog,
    /// Three witness key pairs for 2-of-3 cosigning
    witness_keys: [WitnessKeyPair; 3],
    /// Required signatures (2 of 3)
    threshold: usize,
    /// Signed checkpoints
    checkpoints: Vec<Checkpoint>,
}

/// A Merkle log checkpoint with witness signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub size: u64,
    pub root_hash: Vec<u8>,
    pub signatures: Vec<WitnessSignature>,
    pub timestamp: u64,
}

const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

/// The underlying Merkle log (RFC 6962)
#[derive(Debug, Clone)]
pub struct MerkleLog {
    leaves: Vec<Hash>,
    /// Roots of perfect subtrees that tile the leaves (largest first)
    fringe: VecDeque<(u64, Hash)>,
}

impl MerkleLog {
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            fringe: VecDeque::new(),
        }
    }

    pub fn from_leaves(leaves: Vec<Hash>) -> Self {
        let mut log = Self::new();
        for leaf in leaves {
            log.leaves.push(leaf);
            log.absorb(leaf);
        }
        log
    }

    fn absorb(&mut self, leaf: Hash) {
        self.fringe.push_back((1, leaf));
        while self.fringe.len() >= 2 {
            let len = self.fringe.len();
            let (right_size, right) = self.fringe[fringe.len() - 1];
            let (left_size, left) = self.fringe[fringe.len() - 2];
            if left_size != right_size {
                break;
            }
            let merged = self.node_hash(&left, &right);
            self.fringe.truncate(fringe.len() - 2);
            self.fringe.push_back((left_size + right_size, merged));
        }
    }

    pub fn len(&self) -> u64 {
        self.leaves.len() as u64
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    pub fn leaves(&self) -> &[Hash] {
        &self.leaves
    }

    pub fn append(&mut self, record: &[u8]) -> u64 {
        let leaf = self.leaf_hash(record);
        self.leaves.push(leaf);
        self.absorb(leaf);
        self.leaves.len() as u64 - 1
    }

    fn leaf_hash(&self, record: &[u8]) -> Hash {
        let mut buf = Vec::with_capacity(record.len() + 1);
        buf.push(LEAF_PREFIX);
        buf.extend_from_slice(record);
        blake3::hash(&buf).into()
    }

    fn node_hash(&self, left: &Hash, right: &Hash) -> Hash {
        let mut buf = [0u8; 65];
        buf[0] = NODE_PREFIX;
        buf[1..33].copy_from_slice(left);
        buf[33..65].copy_from_slice(right);
        blake3::hash(&buf).into()
    }

    pub fn root(&self) -> Hash {
        if self.fringe.is_empty() {
            return blake3::hash(&[]).into();
        }
        let mut subtrees = self.fringe.iter().rev();
        let Some((_, rightmost)) = subtrees.next() else {
            return blake3::hash(&[]).into();
        };
        let mut acc = *rightmost;
        for (_, left) in subtrees {
            acc = self.node_hash(left, &acc);
        }
        acc
    }

    pub fn root_hex(&self) -> String {
        hex::encode(self.root())
    }

    fn root_of(&self, leaves: &[Hash]) -> Hash {
        match leaves.len() {
            0 => blake3::hash(&[]).into(),
            1 => leaves[0],
            n => {
                let k = split_point(n);
                let left = self.root_of(&leaves[..k]);
                let right = self.root_of(&leaves[k..]);
                self.node_hash(&left, &right)
            }
        }
    }

    pub fn inclusion_proof(&self, index: u64) -> Option<InclusionProof> {
        if index >= self.len() {
            return None;
        }
        let mut path = Vec::new();
        self.path_into(&self.leaves, index as usize, &mut path);
        Some(InclusionProof {
            leaf_index: index,
            tree_size: self.len(),
            path: path.iter().map(hex::encode).collect(),
        })
    }

    pub fn consistency_proof(&self, first_size: u64) -> Option<ConsistencyProof> {
        let n = self.len();
        if first_size > n {
            return None;
        }
        let mut out = Vec::new();
        if first_size > 0 && first_size < n {
            self.subproof(first_size as usize, &self.leaves, true, &mut out);
        }
        Some(ConsistencyProof {
            first_size,
            second_size: n,
            path: out.iter().map(hex::encode).collect(),
        })
    }

    fn subproof(&self, m: usize, leaves: &[Hash], on_path: bool, out: &mut Vec<Hash>) {
        if m == leaves.len() {
            if !on_path {
                out.push(self.root_of(leaves));
            }
            return;
        }
        let k = split_point(leaves.len());
        if m <= k {
            self.subproof(m, &leaves[..k], on_path, out);
            out.push(self.root_of(&leaves[k..]));
        } else {
            self.subproof(m - k, &leaves[k..], false, out);
            out.push(self.root_of(&leaves[..k]));
        }
    }

    fn path_into(&self, leaves: &[Hash], index: usize, out: &mut Vec<Hash>) {
        if leaves.len() <= 1 {
            return;
        }
        let k = split_point(leaves.len());
        if index < k {
            self.path_into(&leaves[..k], index, out);
            out.push(self.root_of(&leaves[k..]));
        } else {
            self.path_into(&leaves[k..], index - k, out);
            out.push(self.root_of(&leaves[..k]));
        }
    }
}

/// An inclusion proof for a single leaf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    pub leaf_index: u64,
    pub tree_size: u64,
    pub path: Vec<String>, // hex-encoded sibling hashes
}

/// A consistency proof between two tree sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyProof {
    pub first_size: u64,
    pub second_size: u64,
    pub path: Vec<String>, // hex-encoded node hashes
}

fn split_point(n: usize) -> usize {
    debug_assert!(n > 1);
    let mut k = 1;
    while k << 1 < n {
        k <<= 1;
    }
    k
}

impl WitnessMerkleLog {
    pub fn new(witness_keys: [WitnessKeyPair; 3]) -> Self {
        Self {
            merkle: MerkleLog::new(),
            witness_keys,
            threshold: 2,
            checkpoints: Vec::new(),
        }
    }

    pub fn append(&mut self, record: &[u8]) -> u64 {
        self.merkle.append(record)
    }

    pub fn len(&self) -> u64 {
        self.merkle.len()
    }

    pub fn root(&self) -> Hash {
        self.merkle.root()
    }

    pub fn root_hex(&self) -> String {
        self.merkle.root_hex()
    }

    /// Create a checkpoint with 2-of-3 witness signatures
    pub fn checkpoint(&mut self) -> Result<Checkpoint, anyhow::Error> {
        let root = self.root();
        let root_bytes = root.to_vec();
        let mut signatures = Vec::new();

        for (i, key) in self.witness_keys.iter().enumerate() {
            let sig = WitnessSignature::new(i, &root_bytes, &SigningKey::from_bytes(key.signing_key.as_slice().try_into().unwrap()));
            signatures.push(sig);
        }

        // Verify we have at least threshold signatures
        let valid_count = signatures.iter()
            .filter(|sig| sig.verify(&self.verifying_keys()))
            .count();

        if valid_count < self.threshold {
            return Err(anyhow::anyhow!("Insufficient valid witness signatures"));
        }

        let checkpoint = Checkpoint {
            size: self.merkle.len(),
            root_hash: root_bytes,
            signatures,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.checkpoints.push(checkpoint.clone());
        Ok(checkpoint)
    }

    /// Verify a checkpoint's signatures
    pub fn verify_checkpoint(&self, checkpoint: &Checkpoint) -> bool {
        if checkpoint.signatures.len() < self.threshold {
            return false;
        }
        checkpoint.signatures.iter()
            .filter(|sig| sig.verify(&self.verifying_keys()))
            .count() >= self.threshold
    }

    fn verifying_keys(&self) -> [VerifyingKey; 3] {
        [
            VerifyingKey::from_bytes(self.witness_keys[0].verifying_key.as_slice().try_into().unwrap()).unwrap(),
            VerifyingKey::from_bytes(self.witness_keys[1].verifying_key.as_slice().try_into().unwrap()).unwrap(),
            VerifyingKey::from_bytes(self.witness_keys[2].verifying_key.as_slice().try_into().unwrap()).unwrap(),
        ]
    }

    pub fn inclusion_proof(&self, index: u64) -> Option<InclusionProof> {
        self.merkle.inclusion_proof(index)
    }

    pub fn consistency_proof(&self, first_size: u64) -> Option<ConsistencyProof> {
        self.merkle.consistency_proof(first_size)
    }

    pub fn verify_inclusion(
        &self,
        record: &[u8],
        proof: &InclusionProof,
        expected_root_hex: &str,
    ) -> bool {
        if proof.tree_size == 0 || proof.leaf_index >= proof.tree_size {
            return false;
        }

        let mut node = self.merkle.leaf_hash(record);
        let mut fnode = proof.leaf_index;
        let mut snode = proof.tree_size - 1;

        for sibling_hex in &proof.path {
            if snode == 0 {
                return false;
            }
            let Some(sibling) = decode32(sibling_hex) else {
                return false;
            };

            if fnode & 1 == 1 || fnode == snode {
                node = self.merkle.node_hash(&sibling, &node);
                while fnode & 1 == 0 && fnode != 0 {
                    fnode >>= 1;
                    snode >>= 1;
                }
            } else {
                node = self.merkle.node_hash(&node, &sibling);
            }
            fnode >>= 1;
            snode >>= 1;
        }

        if snode != 0 {
            return false;
        }

        hex::encode(node) == expected_root_hex.to_ascii_lowercase()
    }

    pub fn verify_consistency(
        &self,
        first_root_hex: &str,
        second_root_hex: &str,
        proof: &ConsistencyProof,
    ) -> bool {
        let first = proof.first_size;
        let second = proof.second_size;
        if first > second {
            return false;
        }
        if first == second {
            return proof.path.is_empty() && first_root_hex.eq_ignore_ascii_case(second_root_hex);
        }
        if first == 0 {
            return proof.path.is_empty();
        }
        if proof.path.is_empty() {
            return false;
        }

        let Some(path) = proof
            .path
            .iter()
            .map(|h| decode32(h))
            .collect::<Option<Vec<_>>>()
        else {
            return false;
        };

        let mut node = first - 1;
        let mut last = second - 1;

        while node & 1 == 1 {
            node >>= 1;
            last >>= 1;
        }

        let mut idx = 0usize;
        let (mut first_hash, mut second_hash) = if node > 0 {
            idx = 1;
            (path[0], path[0])
        } else {
            let Some(root) = decode32(first_root_hex) else {
                return false;
            };
            (root, root)
        };

        while node > 0 {
            if node & 1 == 1 {
                let Some(sibling) = path.get(idx) else { return false; };
                first_hash = self.merkle.node_hash(sibling, &first_hash);
                second_hash = self.merkle.node_hash(sibling, &second_hash);
                idx += 1;
            } else if node < last {
                let Some(sibling) = path.get(idx) else { return false; };
                second_hash = self.merkle.node_hash(&second_hash, sibling);
                idx += 1;
            }
            node >>= 1;
            last >>= 1;
        }

        if hex::encode(first_hash) != first_root_hex.to_ascii_lowercase() {
            return false;
        }

        while last > 0 {
            let Some(sibling) = path.get(idx) else { return false; };
            second_hash = self.merkle.node_hash(&second_hash, sibling);
            idx += 1;
            last >>= 1;
        }

        idx == path.len() && hex::encode(second_hash) == second_root_hex.to_ascii_lowercase()
    }
}

fn decode32(hex_str: &str) -> Option<Hash> {
    let bytes = hex::decode(hex_str).ok()?;
    let arr: [u8; 32] = bytes.try_into().ok()?;
    Some(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log_of(n: usize) -> WitnessMerkleLog {
        let keys = [
            WitnessKeyPair::generate(),
            WitnessKeyPair::generate(),
            WitnessKeyPair::generate(),
        ];
        let mut log = WitnessMerkleLog::new(keys);
        for i in 0..n {
            log.merkle.append(format!("event-{i}").as_bytes());
        }
        log
    }

    #[test]
    fn empty_tree_has_valid_root() {
        let keys = [WitnessKeyPair::generate(), WitnessKeyPair::generate(), WitnessKeyPair::generate()];
        let log = WitnessMerkleLog::new(keys);
        assert_eq!(
            log.merkle.root_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn witness_can_sign_and_verify() {
        let keys = [WitnessKeyPair::generate(), WitnessKeyPair::generate(), WitnessKeyPair::generate()];
        let log = WitnessMerkleLog::new(keys);
        log.merkle.append(b"test event");
        
        let checkpoint = log.checkpoint().unwrap();
        assert!(log.verify_checkpoint(&checkpoint));
    }

    #[test]
    fn tampered_checkpoint_fails() {
        let keys = [WitnessKeyPair::generate(), WitnessKeyPair::generate(), WitnessKeyPair::generate()];
        let log = WitnessMerkleLog::new(keys);
        log.merkle.append(b"test");
        
        let mut checkpoint = log.checkpoint().unwrap();
        checkpoint.root_hash[0] ^= 0xFF; // Tamper
        assert!(!log.verify_checkpoint(&checkpoint));
    }

    #[test]
    fn inclusion_proof_works() {
        let mut log = log_of(16);
        let proof = log.merkle.inclusion_proof(7).unwrap();
        assert!(log.verify_inclusion(b"event-7", &proof, &log.merkle.root_hex()));
        assert!(!log.verify_inclusion(b"event-7-tampered", &proof, &log.merkle.root_hex()));
    }

    #[test]
    fn consistency_proof_works() {
        let old = log_of(7);
        let new = log_of(12);
        let proof = new.merkle.consistency_proof(7).unwrap();
        assert!(new.verify_consistency(&old.merkle.root_hex(), &new.merkle.root_hex(), &proof));
    }

    #[test]
    fn rewritten_history_detected() {
        let old = log_of(8);
        let mut rewritten = WitnessMerkleLog::new([
            WitnessKeyPair::generate(),
            WitnessKeyPair::generate(),
            WitnessKeyPair::generate(),
        ]);
        for i in 0..12 {
            if i == 3 {
                rewritten.merkle.append(b"event-3-but-altered");
            } else {
                rewritten.merkle.append(format!("event-{i}").as_bytes());
            }
        }
        let proof = rewritten.merkle.consistency_proof(8).unwrap();
        assert!(!rewritten.verify_consistency(&old.merkle.root_hex(), &rewritten.merkle.root_hex(), &proof));
    }
}