//! Test finding bzip2 block markers (π and √π)
//!
//! bzip2 uses two famous 48-bit magic numbers:
//! - Block header: 0x314159265359 (digits of pi: 3.14159265359)
//! - End of stream: 0x177245385090 (digits of sqrt(pi): 1.77245385090)

mod common;

use bitvec::prelude::*;
use common::*;

const BLOCK_MAGIC: u64 = 0x314159265359; // pi
const EOS_MAGIC: u64 = 0x177245385090; // sqrt(pi)

fn find_48bit_markers(data: &[u8], marker: u64) -> Vec<usize> {
    let bits: &BitSlice<u8, Msb0> = BitSlice::from_slice(data);
    let mut found = Vec::new();

    for bit_offset in 0..=(bits.len().saturating_sub(48)) {
        let slice = &bits[bit_offset..bit_offset + 48];
        let mut bytes = [0u8; 6];

        for (i, bit) in slice.iter().enumerate() {
            if *bit {
                let byte_idx = i / 8;
                let bit_idx = 7 - (i % 8);
                bytes[byte_idx] |= 1 << bit_idx;
            }
        }

        let value =
            u64::from_be_bytes([0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]);
        if value == marker {
            found.push(bit_offset);
        }
    }

    found
}

#[test]
fn test_find_bzip2_pi_and_sqrt_pi() {
    let Some(path) = create_bzip2() else { return };

    let data = std::fs::read(&path).expect("Failed to read bz2 file");
    println!(
        "Searching {} bytes ({} bits) for π and √π...",
        data.len(),
        data.len() * 8
    );

    let blocks = find_48bit_markers(&data, BLOCK_MAGIC);
    let eos = find_48bit_markers(&data, EOS_MAGIC);

    for &offset in &blocks {
        println!(
            "✓ Found π (block header) at bit {} (byte {}, +{} bits)",
            offset,
            offset / 8,
            offset % 8
        );
    }
    for &offset in &eos {
        println!(
            "✓ Found √π (EOS) at bit {} (byte {}, +{} bits)",
            offset,
            offset / 8,
            offset % 8
        );
    }

    assert!(!blocks.is_empty(), "Should find at least one π block header");
    assert!(!eos.is_empty(), "Should find √π end-of-stream marker");
}
