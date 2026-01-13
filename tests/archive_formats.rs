//! Tests for various archive format magic numbers
//! Archives are created on-the-fly using system tools

mod common;

use common::*;

// ============================================================================
// gzip tests
// ============================================================================

#[test]
fn test_gzip_magic() {
    let Some(path) = create_gzip() else { return };
    let path_str = path.to_str().unwrap();

    let magic = read_bits_hex(path_str, 0, 16);
    println!("gzip magic (offset 0, 16 bits): {}", magic);
    assert_eq!(magic, "0x1f8b", "Expected gzip magic 0x1F8B");
}

#[test]
fn test_gzip_isize() {
    let Some(path) = create_gzip() else { return };
    let path_str = path.to_str().unwrap();

    let isize = read_bits_decimal_lsb(path_str, -32, 32);
    let original_size = readme_content().len() as u64;

    println!("gzip ISIZE (last 32 bits LE): {}", isize);
    println!("Original content size: {}", original_size);
    assert_eq!(isize, original_size, "gzip ISIZE should match original size");
}

// ============================================================================
// bzip2 tests
// ============================================================================

#[test]
fn test_bzip2_magic() {
    let Some(path) = create_bzip2() else { return };
    let path_str = path.to_str().unwrap();

    let magic = read_bits_hex(path_str, 0, 16);
    println!("bzip2 magic (offset 0, 16 bits): {}", magic);
    assert_eq!(magic, "0x425a", "Expected 'BZ' magic");

    let version = read_bits_hex(path_str, 16, 8);
    println!("bzip2 version (offset 16, 8 bits): {}", version);
    assert_eq!(version, "0x68", "Expected 'h' version");
}

#[test]
fn test_bzip2_pi_marker() {
    let Some(path) = create_bzip2() else { return };
    let path_str = path.to_str().unwrap();

    // Block header magic (pi) starts at bit 32
    let pi = read_bits_hex(path_str, 32, 48);
    println!("bzip2 block magic π (offset 32, 48 bits): {}", pi);
    assert_eq!(pi, "0x314159265359", "Expected π digits");
}

// ============================================================================
// ZIP tests
// ============================================================================

#[test]
fn test_zip_magic() {
    let Some(path) = create_zip() else { return };
    let path_str = path.to_str().unwrap();

    let magic = read_bits_hex(path_str, 0, 32);
    println!("ZIP local header magic (offset 0, 32 bits): {}", magic);
    assert_eq!(magic, "0x504b0304", "Expected PK\\x03\\x04");
}

#[test]
fn test_zip_eocd() {
    let Some(path) = create_zip() else { return };
    let path_str = path.to_str().unwrap();

    // EOCD is 22 bytes from end for minimal ZIP
    let eocd = read_bits_hex(path_str, -176, 32);
    println!("ZIP EOCD magic (offset -176, 32 bits): {}", eocd);
    assert_eq!(eocd, "0x504b0506", "Expected PK\\x05\\x06");
}

// ============================================================================
// tar tests
// ============================================================================

#[test]
fn test_tar_ustar_magic() {
    let Some(path) = create_tar() else { return };
    let path_str = path.to_str().unwrap();

    // ustar magic is at offset 257 bytes = 2056 bits, 5 bytes = "ustar"
    let magic = read_bits_hex(path_str, 2056, 40);
    println!("tar ustar magic (offset 2056, 40 bits): {}", magic);
    // "ustar" = 0x7573746172
    assert_eq!(magic, "0x7573746172", "Expected 'ustar' magic");
}

// ============================================================================
// xz tests
// ============================================================================

#[test]
fn test_xz_magic() {
    let Some(path) = create_tar_xz() else { return };
    let path_str = path.to_str().unwrap();

    let magic = read_bits_hex(path_str, 0, 48);
    println!("xz magic (offset 0, 48 bits): {}", magic);
    assert_eq!(magic, "0xfd377a585a00", "Expected xz magic");
}
