mod bits;
mod cli;
mod display;
mod offset;

use std::fs::File;

use bitvec::prelude::*;
use clap::Parser;
use memmap2::MmapOptions;
use num_bigint::BigUint;

use crate::{
    bits::{extract_bits_to_biguint, extract_bits_to_biguint_lsb},
    cli::{Args, BitOrder, OutputFormat},
    display::{
        best_fit_width, best_fit_width_interleaved, get_terminal_width, print_ascii,
        print_hex_ascii, print_interleaved,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let order = args.order.clone();
    let format = args.format.clone();
    let verbose = args.verbose;
    let (file, offset, length) = args.parse_positional()?;

    let bits = length.to_bits();
    if bits == 0 {
        return Err("Must read at least 1 bit".into());
    }

    let file = File::open(&file)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let file_bits = mmap.len() * 8;

    // Calculate total bits from offset
    let total_bits = offset.to_bits();

    // Resolve negative offset (relative to end of file)
    let offset_bits: usize = if total_bits < 0 {
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

    let end_bit = offset_bits + bits as usize;
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

    if verbose {
        let from_end = file_bits - offset_bits;
        eprintln!("File: {} bytes ({} bits)", mmap.len(), file_bits);
        eprintln!(
            "Reading {} bits at offset {} ({:#x}) = ({} bytes, {} bits) = ({:#x}:{} bits) from end = -{}",
            bits,
            offset_bits,
            offset_bits,
            offset.bytes,
            offset.bits,
            offset.bytes,
            offset.bits,
            from_end
        );
    }

    let value: BigUint = match order {
        BitOrder::Msb => {
            let bits: &BitSlice<u8, Msb0> = BitSlice::from_slice(&mmap[..]);
            extract_bits_to_biguint(&bits[offset_bits..end_bit])
        }
        BitOrder::Lsb => {
            let bits: &BitSlice<u8, Lsb0> = BitSlice::from_slice(&mmap[..]);
            extract_bits_to_biguint_lsb(&bits[offset_bits..end_bit])
        }
    };

    // For text formats, pad bytes to match the requested bit length
    let num_bytes = (bits as usize).div_ceil(8);

    match format {
        OutputFormat::Decimal => println!("{}", value),
        OutputFormat::Hex => println!("{:#x}", value),
        OutputFormat::Binary => println!("{:#b}", value),
        OutputFormat::Ascii => {
            let mut bytes = value.to_bytes_be();
            // Pad with leading zeros if needed
            while bytes.len() < num_bytes {
                bytes.insert(0, 0);
            }
            print_ascii(&bytes);
        }
        OutputFormat::HexAscii => {
            let mut bytes = value.to_bytes_be();
            // Pad with leading zeros if needed
            while bytes.len() < num_bytes {
                bytes.insert(0, 0);
            }

            // Determine width and calculate offset field width
            let term_width = get_terminal_width().unwrap_or(80);

            // Calculate hex digits needed for maximum offset
            let max_offset_bits = offset_bits + bytes.len() * 8;
            let max_offset_bytes = max_offset_bits / 8;
            let offset_hex_width = format!("{:x}", max_offset_bytes).len();

            // Recalculate with real offset width
            let width = best_fit_width(term_width, offset_hex_width as u16);

            print_hex_ascii(&bytes, offset_bits as u64, width as usize, offset_hex_width);
        }
        OutputFormat::Interleaved => {
            let mut bytes = value.to_bytes_be();
            // Pad with leading zeros if needed
            while bytes.len() < num_bytes {
                bytes.insert(0, 0);
            }

            // Determine width and calculate offset field width
            let term_width = get_terminal_width().unwrap_or(80);

            // Calculate hex digits needed for maximum offset
            let max_offset_bits = offset_bits + bytes.len() * 8;
            let max_offset_bytes = max_offset_bits / 8;
            let offset_hex_width = format!("{:x}", max_offset_bytes).len();

            // Recalculate with real offset width
            let width = best_fit_width_interleaved(term_width, offset_hex_width as u16);

            print_interleaved(&bytes, offset_bits as u64, width as usize, offset_hex_width);
        }
    }

    Ok(())
}
