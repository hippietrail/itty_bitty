//! Tests for LLVM IR bitcode (.bc) format and interleaved output format

mod common;

use common::*;

#[test]
fn test_llvm_raw_magic_big_endian() {
    let path = create_raw_llvm_bc();
    let path_str = path.to_str().unwrap();

    // The .bc magic word is bytes 'B', 'C', 0xC0, 0xDE.
    // Read big-endian (MSB first, default), offset 0, 32 bits:
    let magic = read_bits_hex(path_str, 0, 32);
    println!("LLVM bitcode raw magic (offset 0, 32 bits): {}", magic);
    assert_eq!(magic, "0x4243c0de", "Expected LLVM .bc magic 0x4243C0DE");
}

#[test]
fn test_llvm_raw_magic_ascii() {
    let path = create_raw_llvm_bc();
    let path_str = path.to_str().unwrap();

    // In ASCII mode, 'B' and 'C' are printable, while 0xC0 and 0xDE are non-printable (rendered as red squares)
    let ascii_out = read_bits(path_str, 0, 32, "ascii");
    println!("LLVM bitcode raw magic ASCII: {:?}", ascii_out);
    assert!(ascii_out.starts_with("BC"), "Expected ASCII output to start with 'BC'");
    assert!(ascii_out.contains("\x1b[41m \x1b[0m"), "Expected non-printable bytes to use red ANSI squares");
}

#[test]
fn test_llvm_interleaved_format() {
    let path = create_raw_llvm_bc();
    let path_str = path.to_str().unwrap();

    // Test new 2-line interleaved format (-f interleaved)
    let interleaved_out = read_bits(path_str, 0, 32, "interleaved");
    println!("LLVM bitcode interleaved output:\n{}", interleaved_out);

    let lines: Vec<&str> = interleaved_out.lines().collect();
    assert_eq!(lines.len(), 2, "Expected exactly 2 lines for 32-bit (4-byte) interleaved output");

    // Line 1 contains hex representation
    assert!(lines[0].contains("42 43 c0 de"), "Line 1 should contain hex bytes");
    // Line 2 contains ASCII representations 'B', 'C', and red ANSI background squares
    assert!(lines[1].contains("B") && lines[1].contains("C"), "Line 2 should contain ASCII characters B and C");
    assert!(lines[1].contains("\x1b[41m"), "Line 2 should contain red ANSI squares for non-printable 0xC0 and 0xDE");

    // Also test alias hex-ascii-interleaved
    let alias_out = read_bits(path_str, 0, 32, "hex-ascii-interleaved");
    assert_eq!(interleaved_out, alias_out, "Alias hex-ascii-interleaved should match interleaved");
}

#[test]
fn test_llvm_bitstream_variable_width_fields_lsb() {
    let path = create_raw_llvm_bc();
    let path_str = path.to_str().unwrap();

    // LLVM bitstream format specifies bits are read LSB-first within bytes (bit little-endian).
    // Immediately after the 32-bit magic header:
    // 1. ENTER_SUBBLOCK record code is 2 bits, value = 1
    let record_code = read_bits_with_order(path_str, "lsb", "decimal", 32, 2);
    println!("ENTER_SUBBLOCK record code (bit offset 32, 2 bits): {}", record_code);
    assert_eq!(record_code, "1", "Expected ENTER_SUBBLOCK code 1");

    // 2. Block ID is encoded in VBR8 (7 bits value + 1 continuation bit = 8 bits).
    // For IDENTIFICATION_BLOCK_ID (13), bits = 13 (0b0001101) with continuation bit 0 -> 13
    let block_id = read_bits_with_order(path_str, "lsb", "decimal", 34, 8);
    println!("Block ID (bit offset 34, 8 bits): {}", block_id);
    assert_eq!(block_id, "13", "Expected IDENTIFICATION_BLOCK_ID (13)");

    // 3. New abbrev length is encoded in VBR4 (3 bits value + 1 continuation bit = 4 bits).
    // Abbrev length 5 -> 5
    let abbrev_len = read_bits_with_order(path_str, "lsb", "decimal", 42, 4);
    println!("Abbrev length (bit offset 42, 4 bits): {}", abbrev_len);
    assert_eq!(abbrev_len, "5", "Expected new abbrev length 5");

    // 4. Block alignment padding: 18 bits of zeros to pad to 32-bit boundary
    let padding = read_bits_with_order(path_str, "lsb", "decimal", 46, 18);
    println!("32-bit alignment padding (bit offset 46, 18 bits): {}", padding);
    assert_eq!(padding, "0", "Expected padding bits to be 0");

    // 5. Block length in 32-bit words (32 bits, starting at bit 64)
    let block_len = read_bits_with_order(path_str, "lsb", "decimal", 64, 32);
    println!("Block length in words (bit offset 64, 32 bits): {}", block_len);
    assert_eq!(block_len, "7", "Expected block length of 7 words");
}

#[test]
fn test_llvm_clang_bitcode() {
    let Some(path) = create_llvm_bc() else { return };
    let path_str = path.to_str().unwrap();

    // Check if file is Darwin-wrapped (0x0B17C0DE little-endian: 0xDEC0170B)
    let first_32 = read_bits_hex(path_str, 0, 32);
    println!("Clang bitcode first 32 bits: {}", first_32);

    let (magic_bit_offset, stream_bit_offset) = if first_32 == "0xdec0170b" {
        // Wrapped format: raw bitcode starts at byte 20 = bit 160
        println!("Detected Darwin-wrapped LLVM bitcode header");
        (160, 192)
    } else {
        (0, 32)
    };

    let bc_magic = read_bits_hex(path_str, magic_bit_offset, 32);
    println!("LLVM bitcode magic at bit {}: {}", magic_bit_offset, bc_magic);
    assert_eq!(bc_magic, "0x4243c0de", "Expected LLVM magic 0x4243C0DE");

    // Verify first bitstream field (ENTER_SUBBLOCK, 2 bits, LSB)
    let record_code = read_bits_with_order(path_str, "lsb", "decimal", stream_bit_offset, 2);
    println!("ENTER_SUBBLOCK record code at bit {}: {}", stream_bit_offset, record_code);
    assert_eq!(record_code, "1", "Expected ENTER_SUBBLOCK code 1 in real clang bitcode");
}

