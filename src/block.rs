use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct BlockHash(pub Vec<u8>);

impl BlockHash {
    pub fn human(&self) -> String {
        let mut bytes = self.0.clone();
        bytes.reverse();
        hex::encode(bytes)
    }
}

#[derive(Debug, Clone)]
pub struct BlockHeader(pub Vec<u8>);

impl BlockHeader {
    pub fn compute_hash(&self) -> BlockHash {
        BlockHash(compute_sha256(&compute_sha256(&self.0)))
    }
}

fn compute_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
