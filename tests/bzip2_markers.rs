//! Test finding bzip2 block markers
//! 
//! bzip2 uses two famous 48-bit magic numbers:
//! - Block header: 0x314159265359 (digits of pi: 3.14159265359)
//! - End of stream: 0x177245385090 (digits of sqrt(pi): 1.77245385090)

use bitvec::prelude::*;
use std::process::Command;

const BLOCK_MAGIC: u64 = 0x314159265359; // pi
const EOS_MAGIC: u64 = 0x177245385090; // sqrt(pi)

fn read_bits_cli(file: &str, offset: usize, bits: usize) -> u64 {
    let output = Command::new(env!("CARGO_BIN_EXE_itty_bitty"))
        .args([file, &offset.to_string(), &bits.to_string(), "-f", "decimal"])
        .output()
        .expect("Failed to run itty_bitty");

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0)
}

fn find_48bit_markers(data: &[u8], marker: u64) -> Vec<usize> {
    let bits: &BitSlice<u8, Msb0> = BitSlice::from_slice(data);
    let mut found = Vec::new();

    for bit_offset in 0..=(bits.len().saturating_sub(48)) {
        let slice = &bits[bit_offset..bit_offset + 48];
        let num_bytes = 6;
        let mut bytes = [0u8; 6];
        let padding = num_bytes * 8 - 48; // = 0 for 48 bits

        for (i, bit) in slice.iter().enumerate() {
            if *bit {
                let abs_pos = padding + i;
                let byte_idx = abs_pos / 8;
                let bit_idx = 7 - (abs_pos % 8);
                bytes[byte_idx] |= 1 << bit_idx;
            }
        }

        let value = u64::from_be_bytes([0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]);
        if value == marker {
            found.push(bit_offset);
        }
    }

    found
}

#[test]
fn test_bzip2_stream_header() {
    let file = "testdata/README.md.bz2";

    // First 2 bytes: "BZ"
    let magic = read_bits_cli(file, 0, 16);
    println!("Bit 0-15: 'BZ' magic = 0x{:04X}", magic);
    assert_eq!(magic, 0x425A, "Expected 'BZ' magic");

    // Byte 3: version 'h' (0x68) for Huffman
    let version = read_bits_cli(file, 16, 8);
    println!("Bit 16-23: version = 0x{:02X} ('{}')", version, version as u8 as char);
    assert_eq!(version, 0x68, "Expected 'h' version");

    // Byte 4: block size '1'-'9' (100k to 900k)
    let block_size = read_bits_cli(file, 24, 8);
    println!("Bit 24-31: block size = 0x{:02X} ('{}')", block_size, block_size as u8 as char);
    assert!(
        block_size >= 0x31 && block_size <= 0x39,
        "Expected block size '1'-'9', got 0x{:02X}",
        block_size
    );

    println!("\n=> bzip2 stream header: BZh{}", (block_size as u8 - 0x30));
}

#[test]
fn test_bzip2_block_magic_at_known_offset() {
    let file = "testdata/README.md.bz2";

    // Block header starts at bit 32 (byte 4)
    let block_magic = read_bits_cli(file, 32, 48);
    println!("Bit 32-79: block magic = 0x{:012X}", block_magic);
    println!("           expected π  = 0x{:012X}", BLOCK_MAGIC);
    assert_eq!(
        block_magic, BLOCK_MAGIC,
        "Expected block magic 0x{:012X} (pi), got 0x{:012X}",
        BLOCK_MAGIC, block_magic
    );

    println!("\n=> Found π (3.14159265359) at bit 32!");
}

#[test]
fn test_find_all_bzip2_markers() {
    let data = std::fs::read("testdata/README.md.bz2").expect("Failed to read test file");

    println!(
        "Searching {} bytes ({} bits) for pi and sqrt(pi)...",
        data.len(),
        data.len() * 8
    );

    let blocks = find_48bit_markers(&data, BLOCK_MAGIC);
    let eos = find_48bit_markers(&data, EOS_MAGIC);

    for &offset in &blocks {
        println!(
            "  Found pi (block header) at bit {} (byte {}, +{} bits)",
            offset,
            offset / 8,
            offset % 8
        );
    }
    for &offset in &eos {
        println!(
            "  Found sqrt(pi) (EOS) at bit {} (byte {}, +{} bits)",
            offset,
            offset / 8,
            offset % 8
        );
    }

    assert!(!blocks.is_empty(), "Should find at least one block header");
    assert!(!eos.is_empty(), "Should find end-of-stream marker");

    println!("\nSummary:");
    println!(
        "  Block headers (pi): {} found at bits {:?}",
        blocks.len(),
        blocks
    );
    println!(
        "  End of stream (sqrt(pi)): {} found at bits {:?}",
        eos.len(),
        eos
    );
}
