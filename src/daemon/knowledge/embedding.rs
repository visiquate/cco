//! Embedding generation for knowledge items
//!
//! Replicates the JavaScript implementation's SHA256-based embedding generation.
//! This is a deterministic embedding approach where the same text always produces
//! the same vector, enabling consistent similarity searches across sessions.

use sha2::{Digest, Sha256};

/// Embedding dimension (matches JavaScript implementation)
pub const EMBEDDING_DIM: usize = 384;

/// Generate a deterministic embedding from text using SHA256
///
/// This replicates the JavaScript implementation:
/// ```javascript
/// const hash = crypto.createHash('sha256').update(text).digest();
/// for (let i = 0; i < this.embeddingDim; i++) {
///   embedding.push((hash[i % hash.length] / 128.0) - 1.0);
/// }
/// ```
///
/// The embedding:
/// - Is always 384 dimensions
/// - Values are in the range [-1, 1]
/// - Is deterministic (same text = same vector)
/// - Uses SHA256 hash cycling to fill all dimensions
pub fn generate_embedding(text: &str) -> Vec<f32> {
    // Create SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();

    // Convert hash bytes to embedding vector
    let mut embedding = Vec::with_capacity(EMBEDDING_DIM);
    for i in 0..EMBEDDING_DIM {
        // Cycle through hash bytes (32 bytes total)
        let byte_value = hash[i % hash.len()];
        // Normalize to [-1, 1] range: (byte / 128.0) - 1.0
        let normalized = (byte_value as f32 / 128.0) - 1.0;
        embedding.push(normalized);
    }

    embedding
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dimensions() {
        let text = "Test knowledge item";
        let embedding = generate_embedding(text);
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_embedding_range() {
        let text = "Test knowledge item";
        let embedding = generate_embedding(text);

        // All values should be in [-1, 1] range
        for value in embedding.iter() {
            assert!(*value >= -1.0);
            assert!(*value <= 1.0);
        }
    }

    #[test]
    fn test_embedding_deterministic() {
        let text = "Deterministic test";
        let embedding1 = generate_embedding(text);
        let embedding2 = generate_embedding(text);

        // Same text should produce identical embeddings
        assert_eq!(embedding1, embedding2);
    }

    #[test]
    fn test_different_text_different_embedding() {
        let text1 = "First knowledge item";
        let text2 = "Second knowledge item";
        let embedding1 = generate_embedding(text1);
        let embedding2 = generate_embedding(text2);

        // Different text should produce different embeddings
        assert_ne!(embedding1, embedding2);
    }

    #[test]
    fn test_empty_text() {
        let text = "";
        let embedding = generate_embedding(text);
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_long_text() {
        let text = "This is a very long text that contains multiple sentences and should still produce a valid embedding. ".repeat(100);
        let embedding = generate_embedding(&text);
        assert_eq!(embedding.len(), EMBEDDING_DIM);

        // Verify range even with long text
        for value in embedding.iter() {
            assert!(*value >= -1.0);
            assert!(*value <= 1.0);
        }
    }
}
