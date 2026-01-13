# itty_bitty

A CLI tool for reading arbitrary-sized bitfields from files at any bit offset.

## Usage

```
itty_bitty <FILE> <OFFSET> <BITS> [OPTIONS]
```

## Features

- Memory-mapped file access for huge files
- Arbitrary bit widths using BigUint
- MSB/LSB bit ordering
- Multiple output formats: decimal, hex, binary, ascii

