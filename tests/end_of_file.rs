//! Tests for reading fields at the end of files using negative offsets

use std::process::Command;

fn read_bits_cli(file: &str, offset: i64, bits: usize, format: &str, order: &str) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_itty_bitty"));
    cmd.args([file, &offset.to_string(), &bits.to_string()]);
    cmd.args(["-f", format]);
    if order != "msb" {
        cmd.args(["-e", order]);
    }
    let output = cmd.output().expect("Failed to run itty_bitty");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn read_bits_decimal(file: &str, offset: i64, bits: usize, order: &str) -> u64 {
    read_bits_cli(file, offset, bits, "decimal", order)
        .parse()
        .unwrap_or(0)
}

fn read_bits_hex(file: &str, offset: i64, bits: usize) -> String {
    read_bits_cli(file, offset, bits, "hex", "msb")
}

// ============================================================================
// gzip trailer tests
// ============================================================================

#[test]
fn test_gzip_isize_negative_offset() {
    // gzip stores original uncompressed size as last 4 bytes (little-endian)
    let isize = read_bits_decimal("testdata/README.md.gz", -32, 32, "lsb");
    let original_size = std::fs::metadata("testdata/README.md")
        .expect("README.md should exist")
        .len();

    println!("gzip ISIZE (offset -32, 32 bits LE): {}", isize);
    println!("Original README.md size: {}", original_size);

    assert_eq!(
        isize, original_size,
        "gzip ISIZE should match original file size"
    );
}

#[test]
fn test_gzip_crc32_position() {
    // CRC32 is at bytes -8 to -4 (bits -64 to -32)
    let crc_hex = read_bits_cli("testdata/README.md.gz", -64, 32, "hex", "lsb");

    println!("gzip CRC32 (offset -64, 32 bits LE): {}", crc_hex);

    // We can't easily verify the CRC without a crc32 crate, but we can check it's not zero
    assert_ne!(crc_hex, "0x0", "CRC32 should not be zero");
}

// ============================================================================
// ZIP EOCD tests
// ============================================================================

#[test]
fn test_zip_eocd_signature_negative_offset() {
    // ZIP End of Central Directory is at least 22 bytes from end
    // Signature is "PK\x05\x06" = 0x504B0506
    let sig = read_bits_hex("testdata/README.zip", -176, 32); // -22 bytes = -176 bits

    println!("ZIP EOCD signature (offset -176, 32 bits): {}", sig);

    assert_eq!(sig, "0x504b0506", "Expected ZIP EOCD signature PK\\x05\\x06");
}

#[test]
fn test_zip_eocd_entry_count() {
    // Number of entries is at EOCD + 8 bytes (total entries on disk)
    // That's -22 + 10 = -12 bytes from end = -96 bits, 16-bit LE value
    let count = read_bits_decimal("testdata/README.zip", -96, 16, "lsb");

    println!("ZIP total entries (offset -96, 16 bits LE): {}", count);

    assert_eq!(count, 1, "Our test ZIP should have exactly 1 entry");
}

// ============================================================================
// Negative offset edge cases
// ============================================================================

#[test]
fn test_negative_offset_exact_file_start() {
    // -N where N = file size in bits should give us bit 0
    let file_bits = std::fs::metadata("testdata/README.md.bz2").unwrap().len() as i64 * 8;

    // Read first 16 bits using negative offset
    let via_negative = read_bits_hex("testdata/README.md.bz2", -file_bits, 16);
    let via_positive = read_bits_hex("testdata/README.md.bz2", 0, 16);

    println!("First 16 bits via offset 0: {}", via_positive);
    println!("First 16 bits via offset -{}: {}", file_bits, via_negative);

    assert_eq!(via_negative, via_positive);
    assert_eq!(via_positive, "0x425a", "Should be 'BZ' magic");
}

#[test]
fn test_negative_offset_last_byte() {
    // Read last 8 bits
    let last_byte = read_bits_hex("testdata/README.md.bz2", -8, 8);
    println!("Last byte of bz2 (offset -8, 8 bits): {}", last_byte);

    // Just verify we got something (the actual value depends on the compressed data)
    assert!(last_byte.starts_with("0x"), "Should return hex value");
}
