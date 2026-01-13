# itty-bitty

A CLI tool for reading arbitrary-sized bitfields from files at any bit offset.

## Features

- **Memory-mapped file access** — handles gigantic files without loading into RAM
- **Arbitrary bit widths** — read 1 bit or 1000 bits, no 64-bit limit
- **Bit-level precision** — specify exact bit offsets, not just bytes
- **Negative offsets** — read from end of file (e.g., `-32` for last 32 bits)
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
| `OFFSET` | Bit offset (negative = from end of file) |
| `BITS` | Number of bits to read |

### Options

| Option | Description |
|--------|-------------|
| `-e, --order <ORDER>` | Bit order: `msb` (default) or `lsb` |
| `-f, --format <FORMAT>` | Output: `hex` (default), `decimal`, `binary`, `ascii` |
| `-v, --verbose` | Show offset info (both from start and from end) |

## Examples

Read the first byte of a file:
```bash
itty-bitty myfile.bin 0 8
# 0x42
```

Read 48 bits starting at bit offset 32:
```bash
itty-bitty archive.bz2 32 48
# 0x314159265359  (that's π!)
```

Read 3 bits at a non-byte-aligned offset:
```bash
itty-bitty data.bin 5 3 -f decimal
# Returns value 0-7
```

Read the last 32 bits of a file (gzip stores original size here):
```bash
itty-bitty README.md.gz -32 32 -e lsb -f decimal
# 329
```

Use verbose mode to see both positive and negative offsets:
```bash
itty-bitty archive.bz2 2130 48 -v
# File: 277 bytes (2216 bits)
# Reading 48 bits at offset 2130 (byte 266, +2 bits) = -86 from end
# 0x177245385090
```

## Fun Fact: Finding π in bzip2 Files

bzip2 uses the digits of π (3.14159265359) and √π (1.77245385090) as 48-bit magic markers:

```bash
# Block header marker (π)
itty-bitty README.md.bz2 32 48
# 0x314159265359

# End-of-stream marker (√π) — often at an unaligned bit offset!
itty-bitty README.md.bz2 -86 48
# 0x177245385090
```

The tests include a bit-scanner that finds these markers at arbitrary bit positions.

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
