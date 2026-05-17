pub fn hash_bin(s: &str) -> u32 {
    s.to_ascii_lowercase().bytes().fold(0x811c9dc5_u32, |h, b| {
        (h ^ b as u32).wrapping_mul(0x01000193)
    })
}
