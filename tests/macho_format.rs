//! Tests for Mach-O executable format fields
//! Uses our own release binary as the test subject

mod common;

use common::*;
use std::path::Path;

fn get_macho_binary() -> Option<&'static str> {
    let path = "target/release/itty-bitty";
    if Path::new(path).exists() {
        Some(path)
    } else {
        println!("⏭ Release binary not found, run `cargo build --release` first");
        None
    }
}

// ============================================================================
// Mach-O header tests (64-bit ARM64)
// ============================================================================

#[test]
fn test_macho_magic() {
    let Some(path) = get_macho_binary() else { return };

    // Magic number at offset 0, 32 bits
    // 0xFEEDFACF = 64-bit Mach-O (stored as little-endian on disk: CF FA ED FE)
    // When read as big-endian (default), we see 0xCFFAEDFE
    let magic = read_bits_hex(path, 0, 32);
    println!("Mach-O magic (offset 0, 32 bits): {}", magic);
    assert_eq!(magic, "0xcffaedfe", "Expected 64-bit Mach-O magic (LE on disk)");
}

#[test]
fn test_macho_cpu_type() {
    let Some(path) = get_macho_binary() else { return };

    // CPU type at offset 4 bytes = 32 bits, 32 bits wide
    // 0x0100000C = CPU_TYPE_ARM64 (little-endian: 0x0C000001)
    let cpu_type = read_bits_decimal_lsb(path, 32, 32);
    println!("Mach-O CPU type (offset 32 bits, 32 bits LE): {}", cpu_type);
    // ARM64 = 0x0100000C = 16777228
    assert_eq!(cpu_type, 16777228, "Expected ARM64 CPU type");
}

#[test]
fn test_macho_file_type() {
    let Some(path) = get_macho_binary() else { return };

    // File type at offset 12 bytes = 96 bits, 32 bits wide
    // MH_EXECUTE = 0x02
    let file_type = read_bits_decimal_lsb(path, 96, 32);
    println!("Mach-O file type (offset 96 bits, 32 bits LE): {}", file_type);
    assert_eq!(file_type, 2, "Expected MH_EXECUTE file type");
}
