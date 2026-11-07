# itty-bitty

A CLI tool for reading arbitrary-sized bitfields from files at any bit offset.

## Features

- **Memory-mapped file access** — handles gigantic files without loading into RAM
- **Arbitrary bit widths** — read 1 bit or 1000 bits, no 64-bit limit
- **Bit-level precision** — specify exact bit offsets, not just bytes
- **Flexible offset syntax** — hex, decimal, byte+bit, negative offsets
- **MSB/LSB ordering** — supports both bit orderings
- **Multiple output formats** — hex (default), decimal, binary, ASCII, hex-ascii, interleaved (2-line hex/ASCII)

## Installation

```bash
cargo install --path .
```

## Usage

**Important:** Options (like `-f hex-ascii`) must come **before** positional arguments.

```
itty-bitty [OPTIONS] <OFFSET> <LENGTH> <FILE>
itty-bitty [OPTIONS] <FILE> <OFFSET> <LENGTH>
```

Both orderings are supported — choose what's convenient. **Options must come first.**

### Quick Examples

```bash
# Hexdump-style view of first 256 bytes (most common use case)
itty-bitty -f hex-ascii myfile.bin 0 2048

# Read last 32 bits
itty-bitty -f hex-ascii myfile.bin -32 32

# Read 48 bits at a specific offset
itty-bitty archive.bz2 32 48
# 0x314159265359  (that's π!)
```

### Arguments

| Argument | Description |
|----------|-------------|
| `FILE` | Input file path |
| `OFFSET` | Bit/byte offset with optional bits (see below) |
| `LENGTH` | Number of bits to read |

### Offset Syntax

The `OFFSET` argument supports several flexible formats:

**Basic Formats**
- Decimal: 1234
- Hex: 0x4D2 or $4D2 or 4D2h
- With thousands separators: 1_234_567 or 1,234,567

**Byte + Bit Offsets**
- Colon or dot notation: 123:4 or 123.4 (123 bytes + 4 bits)
- Hex with bits: 0x1A:3 (0x1A bytes + 3 bits)

**From End of File**
- Negative offset: -32 (last 32 bits)
- Negative hex: -0x10 (last 16 bytes)
- Negative byte+bit: -1024:4 (1024 bytes + 4 bits from end)

### Options

| Option | Description |
|--------|-------------|
| `-e, --order <ORDER>` | Bit order: `msb` (default) or `lsb` |
| `-f, --format <FORMAT>` | Output: `hex` (default), `decimal`, `binary`, `ascii`, `hex-ascii`, `interleaved` |
| `-v, --verbose` | Show detailed offset information |

> **Note on Endianness:** `itty-bitty` currently controls bit stream ordering (`-e msb` or `-e lsb`) across the entire requested range rather than byte-oriented word/dword/qword endianness. However, you can achieve a similar result when reading 16-bit, 32-bit, or 64-bit little-endian integers on byte boundaries by using `-e lsb`, because it evaluates bits starting from bit 0 of the first byte up to bit 7 of the last byte (`byte[0] | (byte[1] << 8) | ...`).

### Output Formats

| Format | Description | Example |
|--------|-------------|---------|
| `hex` | Single line of hex bytes | `0x4d5a7146...` |
| `decimal` | Single line decimal number | `1639894643...` |
| `binary` | Single line binary | `0b11000000110001...` |
| `ascii` | Single line ASCII (non-printable shown as red blocks) | `MZqFpD='<red block>` |
| `hex-ascii` | Hexdump-style: offset + hex + ASCII (one line per row) | See example below |
| `interleaved` | 2-line interleaved: hex line on top, ASCII/red block below | See example below |

**hex-ascii example:**
```
00: 4d 5a 71 46 70 44 3d 27 0a 00 00 10 00 f8 00 00 00 00 00 00 00 01 00 08 40 00 00 00 00 00 00 00 | MZqFpD='                @
10: 2c 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 | ,                               |
```

**interleaved example:**
```
00: 42 43 c0 de 35 14 00 00 48 65 6c 6c 6f 20 21 0a 
     B  C        5           H  e  l  l  o     !    
```

## Examples

### Basic Usage
```bash
# Read first byte
itty-bitty myfile.bin 0 8
# Read 48 bits starting at bit offset 32
itty-bitty archive.bz2 32 48
# 0x314159265359  (that's π!)
```

### Flexible Argument Ordering
```bash
# Same command, two ways:
itty-bitty file.bin 100 64         # FILE OFFSET LENGTH
itty-bitty 100 64 file.bin         # OFFSET LENGTH FILE

# The second form makes it easy to test multiple files without retyping:
for f in *.bin; do itty-bitty 100 64 "$f"; done
```

### Hex Offsets
```bash
# Hex with 0x prefix
itty-bitty data.bin 0x1A 8
# Hex with $ prefix
itty-bitty data.bin $1A 8
# Hex with h suffix
itty-bitty data.bin 1Ah 8
```

### Byte + Bit Offsets
```bash
# 123 bytes + 4 bits
itty-bitty file.bin 123:4 8
# Hex with bits
itty-bitty file.bin 0x1A.3 5
```

### From End of File
```bash
# Last 32 bits
itty-bitty file.bin -32 32
# Last 0x100 bytes + 3 bits
itty-bitty file.bin -0x100:3 5
```

### Verbose Mode
```bash
itty-bitty archive.bz2 0x200.3 16 -v
# File: 1024 bytes (8192 bits)
# Reading 16 bits at offset 4099 (0x1000 bytes, 3 bits) = -4093 from end
# 0xABCD
```

### Hex-Ascii Mode (Hexdump-style)
```bash
# View file start in hexdump format
itty-bitty -f hex-ascii myfile.bin 0 256
# 00: 4d 5a 90 00 03 00 00 00 04 00 00 00 ff ff 00 00 | MZ..............
# 10: b8 00 00 00 00 00 00 00 40 00 00 00 00 00 00 00 | ....@.......

# View end of file
itty-bitty -f hex-ascii myfile.bin -256 256
```

### Interleaved Mode (2-line Hex/ASCII)
```bash
# View file in 2-line interleaved format (hex with ASCII aligned directly underneath)
itty-bitty -f interleaved myfile.bin 0 128
# 00: 42 43 c0 de 35 14 00 00 48 65 6c 6c 6f 20 21 0a 
#      B  C        5           H  e  l  l  o     !    
```

### Little-Endian Integers (Word / Dword / Qword)
While byte-oriented word/dword/qword endianness is not yet a separate option, using `-e lsb` evaluates multi-byte integers on byte boundaries in little-endian order:
```bash
# Read a 32-bit little-endian integer (dword) in decimal
itty-bitty -e lsb -f decimal myfile.bin 0 32

# Read a 16-bit little-endian integer (word) in hex
itty-bitty -e lsb -f hex myfile.bin 0 16
```

## Implementation

Built with:
- [`memmap2`](https://docs.rs/memmap2) — memory-mapped file I/O
- [`bitvec`](https://docs.rs/bitvec) — bit-level slice operations
- [`num-bigint`](https://docs.rs/num-bigint) — arbitrary-precision integers
- [`clap`](https://docs.rs/clap) — CLI argument parsing

## License

MIT
