use bitvec::prelude::*;
use clap::{Parser, ValueEnum};
use memmap2::MmapOptions;
use num_bigint::BigUint;
use std::{fs::File, str::FromStr};

#[derive(Debug)]
enum OffsetError {
    ParseError(String),
    InvalidBitOffset,
}

impl std::fmt::Display for OffsetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OffsetError::ParseError(msg) => write!(f, "Invalid offset: {}", msg),
            OffsetError::InvalidBitOffset => write!(f, "Bit offset must be 0-7"),
        }
    }
}

impl std::error::Error for OffsetError {}

fn parse_offset(s: &str) -> Result<Offset, String> {
    Offset::from_str(s).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
struct Offset {
    bytes: u64,
    bits: u32,  // 0-7
    is_negative: bool,
}

impl Offset {
    fn to_bits(&self) -> i64 {
        let total_bits = (self.bytes * 8) as i64 + self.bits as i64;
        if self.is_negative { -total_bits } else { total_bits }
    }
}

impl FromStr for Offset {
    type Err = OffsetError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (sign, s) = match s.strip_prefix('-') {
            Some(rest) => (true, rest.trim_start()),
            None => (false, s.trim_start()),
        };
        
        // Remove thousands separators
        let s = s.replace(&[',', '_', '\''][..], "");
        
        // Try to split into bytes:bits (or bytes.bits)
        let (num_str, bits_part) = if let Some((a, b)) = s.split_once(|c| c == ':' || c == '.') {
            let bits = b.parse::<u32>()
                .map_err(|_| OffsetError::ParseError("Invalid bit count".into()))?;
            if bits > 7 {
                return Err(OffsetError::InvalidBitOffset);
            }
            (a, Some(bits))
        } else {
            (s.as_str(), None)
        };
        
        // Parse the number part
        let num = if num_str.starts_with("0x") || num_str.starts_with("0X") {
            u64::from_str_radix(&num_str[2..], 16)
        } else if num_str.starts_with('$') {
            u64::from_str_radix(&num_str[1..], 16)
        } else if num_str.ends_with('h') || num_str.ends_with('H') {
            u64::from_str_radix(&num_str[..num_str.len()-1], 16)
        } else {
            num_str.parse::<u64>()
        }.map_err(|e| OffsetError::ParseError(e.to_string()))?;
        
        // If bits were specified via colon/dot, treat num as bytes
        // Otherwise, treat num as total bits
        let (bytes, bits) = if let Some(bit_offset) = bits_part {
            (num, bit_offset)
        } else {
            (num / 8, (num % 8) as u32)
        };
        
        Ok(Offset {
            bytes,
            bits,
            is_negative: sign,
        })
    }
}

#[derive(Clone, ValueEnum, Debug)]
enum BitOrder {
    Msb,
    Lsb,
}

#[derive(Parser)]
#[command(about = "Read an arbitrary-sized bitfield from a file at any bit offset")]
#[command(arg_required_else_help = true)]
struct Args {
    /// Input file path
    file: String,

    /// Bit/byte offset with optional bits (e.g., '123', '0x1A:3', '1_000.5')
    /// Supports hex (0x, $, or h suffix) and thousands separators (_, ',', ' ')
    /// Negative values count from end of file
    #[arg(value_parser = parse_offset)]
    offset: Offset,

    /// Number of bits to read
    bits: usize,

    /// Bit order (msb = most significant bit first, lsb = least significant bit first)
    #[arg(short = 'e', long, value_enum, default_value = "msb")]
    order: BitOrder,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "hex")]
    format: OutputFormat,

    /// Show offset info (both from start and from end)
    #[arg(short = 'v', long)]
    verbose: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Decimal,
    Hex,
    Binary,
    Ascii,
    HexAscii,
}

fn extract_bits_to_biguint(bits: &BitSlice<u8, Msb0>) -> BigUint {
    let n = bits.len();
    if n == 0 {
        return BigUint::ZERO;
    }

    let num_bytes = (n + 7) / 8;
    let mut bytes = vec![0u8; num_bytes];
    let padding = num_bytes * 8 - n;

    for (i, bit) in bits.iter().enumerate() {
        if *bit {
            let abs_pos = padding + i;
            let byte_idx = abs_pos / 8;
            let bit_idx = 7 - (abs_pos % 8);
            bytes[byte_idx] |= 1 << bit_idx;
        }
    }

    BigUint::from_bytes_be(&bytes)
}

fn extract_bits_to_biguint_lsb(bits: &BitSlice<u8, Lsb0>) -> BigUint {
    let mut result = BigUint::ZERO;
    for (i, bit) in bits.iter().enumerate() {
        if *bit {
            result |= BigUint::from(1u8) << i;
        }
    }
    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.bits == 0 {
        return Err("Must read at least 1 bit".into());
    }

    let file = File::open(&args.file)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let file_bits = mmap.len() * 8;

    // Calculate total bits from offset
    let total_bits = args.offset.to_bits();
    
    // Resolve negative offset (relative to end of file)
    let offset: usize = if total_bits < 0 {
        let from_end = (-total_bits) as usize;
        if from_end > file_bits {
            return Err(format!(
                "Negative offset -{} exceeds file size ({} bits)",
                from_end, file_bits
            )
            .into());
        }
        file_bits - from_end
    } else {
        total_bits as usize
    };

    let end_bit = offset + args.bits;
    if end_bit > file_bits {
        let excess_bits = end_bit - file_bits;
        return Err(format!(
            "Requested range exceeds file size: need bit {}, but file is {} bytes ({} bits) — {} bits past end",
            end_bit - 1,
            mmap.len(),
            file_bits,
            excess_bits
        )
        .into());
    }

    if args.verbose {
        let from_end = file_bits - offset;
        eprintln!(
            "File: {} bytes ({} bits)",
            mmap.len(),
            file_bits
        );
        eprintln!(
            "Reading {} bits at offset {} ({:#x}) = ({} bytes, {} bits) = ({:#x}:{} bits) from end = -{}",
            args.bits,
            offset,
            offset,
            args.offset.bytes,
            args.offset.bits,
            args.offset.bytes,
            args.offset.bits,
            from_end
        );
    }

    let value: BigUint = match args.order {
        BitOrder::Msb => {
            let bits: &BitSlice<u8, Msb0> = BitSlice::from_slice(&mmap[..]);
            extract_bits_to_biguint(&bits[offset..end_bit])
        }
        BitOrder::Lsb => {
            let bits: &BitSlice<u8, Lsb0> = BitSlice::from_slice(&mmap[..]);
            extract_bits_to_biguint_lsb(&bits[offset..end_bit])
        }
    };

    match args.format {
        OutputFormat::Decimal => println!("{}", value),
        OutputFormat::Hex => println!("{:#x}", value),
        OutputFormat::Binary => println!("{:#b}", value),
        OutputFormat::Ascii => print_ascii(&value.to_bytes_be()),
        OutputFormat::HexAscii => print_hex_ascii(&value.to_bytes_be()),
    }

    Ok(())
}

fn print_ascii(bytes: &[u8]) {
    for &b in bytes {
        if b.is_ascii_graphic() || b == b' ' {
            print!("{}", b as char);
        } else {
            // ANSI: red background for non-printable
            print!("\x1b[41m \x1b[0m");
        }
    }
    println!();
}

fn print_hex_ascii(bytes: &[u8]) {
    // Print hex bytes
    for &b in bytes {
        print!("{:02x} ", b);
    }
    
    // Padding if less than 16 bytes
    if bytes.len() < 16 {
        for _ in bytes.len()..16 {
            print!("   ");
        }
    }
    
    // Separator
    print!(" | ");
    
    // Print ASCII
    for &b in bytes {
        if b.is_ascii_graphic() || b == b' ' {
            print!("{}", b as char);
        } else {
            // ANSI: red background for non-printable
            print!("\x1b[41m \x1b[0m");
        }
    }
    println!();
}
