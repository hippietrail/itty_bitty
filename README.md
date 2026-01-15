# itty-bitty

A CLI tool for reading arbitrary-sized bitfields from files at any bit offset.

## Features

- **Memory-mapped file access** — handles gigantic files without loading into RAM
- **Arbitrary bit widths** — read 1 bit or 1000 bits, no 64-bit limit
- **Bit-level precision** — specify exact bit offsets, not just bytes
- **Flexible offset syntax** — hex, decimal, byte+bit, negative offsets
- **MSB/LSB ordering** — supports both bit orderings
- **Multiple output formats** — hex (default), decimal, binary, ASCII

## Installation

```bash
cargo install --path .
```

## Usage

```
itty-bitty <FILE> <OFFSET> <BITS> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `FILE` | Input file path |
| `OFFSET` | Bit/byte offset with optional bits (see below) |
| `BITS` | Number of bits to read |

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
| `-f, --format <FORMAT>` | Output: `hex` (default), `decimal`, `binary`, `ascii` |
| `-v, --verbose` | Show detailed offset information |

## Examples

### Basic Usage
```bash
# Read first byte
itty-bitty myfile.bin 0 8
# Read 48 bits starting at bit offset 32
itty-bitty archive.bz2 32 48
# 0x314159265359  (that's π!)
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

## Implementation

Built with:
- [`memmap2`](https://docs.rs/memmap2) — memory-mapped file I/O
- [`bitvec`](https://docs.rs/bitvec) — bit-level slice operations
- [`num-bigint`](https://docs.rs/num-bigint) — arbitrary-precision integers
- [`clap`](https://docs.rs/clap) — CLI argument parsing
- [Amp Free](https://ampcode.com/news/amp-free) — free AI coding agent support by ads
- [Beads](https://github.com/steveyegge/beads) — distributed, git-backed graph issue tracker for AI agents

## License

MIT
