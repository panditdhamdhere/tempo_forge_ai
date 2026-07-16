//! Deterministic local embeddings for TempoForge RAG.
//! Production can swap this for OpenAI/Groq embeddings behind the same trait.

pub const EMBED_DIM: usize = 256;

pub fn embed_text(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0f32; EMBED_DIM];
    let lower = text.to_lowercase();
    for token in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
        if token.len() < 3 {
            continue;
        }
        let mut hash: u64 = 1469598103934665603;
        for b in token.bytes() {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(1099511628211);
        }
        let idx = (hash as usize) % EMBED_DIM;
        let sign = if hash & 1 == 0 { 1.0 } else { -1.0 };
        vector[idx] += sign;
    }

    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-6);
    for v in &mut vector {
        *v /= norm;
    }
    vector
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similar_texts_share_signal() {
        let a = embed_text("Tempo TIP-20 fee token balanceOf");
        let b = embed_text("TIP-20 balanceOf on Tempo chain");
        let c = embed_text("completely unrelated cooking recipe");
        let sim_ab: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let sim_ac: f32 = a.iter().zip(c.iter()).map(|(x, y)| x * y).sum();
        assert!(sim_ab > sim_ac);
    }
}
