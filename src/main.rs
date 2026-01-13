use bitvec::prelude::*;
use clap::{Parser, ValueEnum};
use memmap2::MmapOptions;
use num_bigint::BigUint;
use std::fs::File;

#[derive(Clone, ValueEnum)]
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

    /// Bit offset (negative = from end of file, e.g., -32 means last 32 bits)
    #[arg(allow_negative_numbers = true)]
    offset: i64,

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

    // Resolve negative offset (relative to end of file)
    let offset: usize = if args.offset < 0 {
        let from_end = (-args.offset) as usize;
        if from_end > file_bits {
            return Err(format!(
                "Negative offset -{} exceeds file size ({} bits)",
                from_end, file_bits
            )
            .into());
        }
        file_bits - from_end
    } else {
        args.offset as usize
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
            "Reading {} bits at offset {} (byte {}, +{} bits) = -{} from end",
            args.bits,
            offset,
            offset / 8,
            offset % 8,
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
