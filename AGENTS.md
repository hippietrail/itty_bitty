# Agent TODO

- [ ] **Word / Dword / Qword Endianness**: Consider how we might implement byte-oriented endianness (16-bit word, 32-bit dword, 64-bit qword) in addition to the current stream bit-ordering (`-e msb` / `-e lsb`). Explore CLI syntax (e.g. `--endian <le|be>`, `--word-size <16|32|64>`, or length/type annotations like `32le`) and how to handle mixed endianness (little-endian byte streams containing MSB-first bitfields).
