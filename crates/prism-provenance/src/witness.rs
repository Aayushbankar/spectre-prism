use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use std::fs;
use std::path::Path;

pub struct WitnessKey {
    pub signing_key: SigningKey,
}

pub struct WitnessQuorum {
    pub keys: [WitnessKey; 3],
}

impl WitnessKey {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        WitnessKey { signing_key }
    }

    pub fn public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        WitnessKey {
            signing_key: SigningKey::from_bytes(bytes),
        }
    }
}

pub fn sign_checkpoint(root_hash: &[u8; 32], witness: &WitnessKey) -> Signature {
    witness.signing_key.sign(root_hash)
}

pub fn verify_quorum(
    root_hash: &[u8; 32],
    signatures: &[(VerifyingKey, Signature)],
    threshold: usize,
) -> bool {
    let mut seen_keys = std::collections::HashSet::new();
    let valid_count = signatures
        .iter()
        .filter(|(pk, sig)| {
            if seen_keys.contains(&pk.to_bytes()) {
                return false;
            }
            if pk.verify(root_hash, sig).is_ok() {
                seen_keys.insert(pk.to_bytes());
                true
            } else {
                false
            }
        })
        .count();
    valid_count >= threshold
}

pub fn generate_witness_keys() -> (WitnessKey, WitnessKey, WitnessKey) {
    (
        WitnessKey::generate(),
        WitnessKey::generate(),
        WitnessKey::generate(),
    )
}

pub fn save_keys(keys: &(WitnessKey, WitnessKey, WitnessKey), dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("witness1.key"), keys.0.to_bytes())?;
    fs::write(dir.join("witness2.key"), keys.1.to_bytes())?;
    fs::write(dir.join("witness3.key"), keys.2.to_bytes())?;
    Ok(())
}

pub fn load_keys(dir: &Path) -> std::io::Result<(WitnessKey, WitnessKey, WitnessKey)> {
    let k1 = fs::read(dir.join("witness1.key"))?;
    let k2 = fs::read(dir.join("witness2.key"))?;
    let k3 = fs::read(dir.join("witness3.key"))?;

    let to_arr = |vec: Vec<u8>| {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&vec[..32]);
        arr
    };

    Ok((
        WitnessKey::from_bytes(&to_arr(k1)),
        WitnessKey::from_bytes(&to_arr(k2)),
        WitnessKey::from_bytes(&to_arr(k3)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_sign_and_verify() {
        let (k1, k2, k3) = generate_witness_keys();
        let root_hash = [1u8; 32];

        let sig1 = sign_checkpoint(&root_hash, &k1);
        let sig2 = sign_checkpoint(&root_hash, &k2);
        let sig3 = sign_checkpoint(&root_hash, &k3);

        let signatures = vec![
            (k1.public_key(), sig1),
            (k2.public_key(), sig2),
            (k3.public_key(), sig3),
        ];

        assert!(verify_quorum(&root_hash, &signatures, 2));
    }

    #[test]
    fn test_witness_single_signer_fails() {
        let (k1, _k2, _k3) = generate_witness_keys();
        let root_hash = [1u8; 32];

        let sig1 = sign_checkpoint(&root_hash, &k1);

        let signatures = vec![(k1.public_key(), sig1)];

        assert!(!verify_quorum(&root_hash, &signatures, 2));
    }

    #[test]
    fn test_witness_tampered_root_fails() {
        let (k1, k2, _k3) = generate_witness_keys();
        let root_hash = [1u8; 32];

        let sig1 = sign_checkpoint(&root_hash, &k1);
        let sig2 = sign_checkpoint(&root_hash, &k2);

        let tampered_hash = [2u8; 32];

        let signatures = vec![
            (k1.public_key(), sig1),
            (k2.public_key(), sig2),
        ];

        assert!(!verify_quorum(&tampered_hash, &signatures, 2));
    }

    #[test]
    fn test_witness_key_persistence() {
        let dir = std::env::temp_dir().join(format!("witness_test_{}", rand::random::<u32>()));
        let keys = generate_witness_keys();
        save_keys(&keys, &dir).unwrap();

        let loaded = load_keys(&dir).unwrap();
        let root_hash = [3u8; 32];
        
        let sig1 = sign_checkpoint(&root_hash, &loaded.0);
        let signatures = vec![(loaded.0.public_key(), sig1)];

        assert!(verify_quorum(&root_hash, &signatures, 1));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
