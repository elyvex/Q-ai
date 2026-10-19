//! Embedding providers for the multi-RAG platform.
//!
//! # embeddings
//!
//! Local, deterministic defaults only in this slice — no network.

use sha2::{Digest, Sha256};

/// Embedding provider trait.
pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn dimension(&self) -> usize;
    fn embed(&self, text: &str) -> Vec<f32>;
}

/// Deterministic local hash-based embedding (no network).
#[derive(Debug, Clone)]
pub struct LocalHashEmbedding {
    dim: usize,
}

impl LocalHashEmbedding {
    pub fn new(dim: usize) -> Self {
        Self { dim: dim.max(1) }
    }
}

impl Default for LocalHashEmbedding {
    fn default() -> Self {
        Self::new(128)
    }
}

impl EmbeddingProvider for LocalHashEmbedding {
    fn name(&self) -> &'static str {
        "local-hash"
    }
    fn dimension(&self) -> usize {
        self.dim
    }
    fn embed(&self, text: &str) -> Vec<f32> {
        let mut out = vec![0.0f32; self.dim];
        // Expand sha256 over text + index into dim floats in [-1,1].
        let mut counter = 0u64;
        for i in 0..self.dim {
            if i % 8 == 0 {
                let mut h = Sha256::new();
                h.update(text.as_bytes());
                h.update(counter.to_le_bytes());
                let digest = h.finalize();
                for (j, b) in digest.iter().enumerate() {
                    if i + j < self.dim {
                        out[i + j] = (*b as f32 / 127.5) - 1.0;
                    }
                }
                counter += 1;
            }
        }
        normalize(&mut out);
        out
    }
}

fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// Cosine similarity.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..n {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

/// Serialize f32 slice to little-endian bytes.
pub fn to_blob(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

/// Deserialize little-endian bytes into f32 vec.
pub fn from_blob(bytes: &[u8], dim: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(dim);
    for chunk in bytes.chunks_exact(4).take(dim) {
        out.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let p = LocalHashEmbedding::new(128);
        assert_eq!(p.embed("hello"), p.embed("hello"));
    }

    #[test]
    fn cosine_identical_is_one() {
        let p = LocalHashEmbedding::new(128);
        let v = p.embed("abc");
        let c = cosine(&v, &v);
        assert!((c - 1.0).abs() < 1e-6);
    }

    #[test]
    fn blob_roundtrip() {
        let p = LocalHashEmbedding::new(32);
        let v = p.embed("x");
        let b = to_blob(&v);
        let back = from_blob(&b, 32);
        assert_eq!(v, back);
    }
}
