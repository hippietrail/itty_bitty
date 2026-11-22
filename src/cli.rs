use std::str::FromStr;

use clap::{Parser, ValueEnum};

use crate::offset::{Length, Offset};

#[derive(Clone, ValueEnum, Debug)]
pub(crate) enum BitOrder {
    Msb,
    Lsb,
}

#[derive(Parser)]
#[command(about = "Read an arbitrary-sized bitfield from a file at any bit offset")]
#[command(
    override_usage = "itty-bitty [OPTIONS] <OFFSET> <LENGTH> <FILE>\n    itty-bitty [OPTIONS] <FILE> <OFFSET> <LENGTH>"
)]
#[command(arg_required_else_help = true)]
pub(crate) struct Args {
    /// Positional arguments: [OFFSET LENGTH FILE] or [FILE OFFSET LENGTH]
    #[arg(trailing_var_arg = true, hide = true)]
    positional: Vec<String>,

    /// Bit order (msb = most significant bit first, lsb = least significant bit first)
    #[arg(short = 'e', long, value_enum, default_value = "msb")]
    pub(crate) order: BitOrder,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "hex")]
    pub(crate) format: OutputFormat,

    /// Show offset info (both from start and from end)
    #[arg(short = 'v', long)]
    pub(crate) verbose: bool,
}

impl Args {
    pub(crate) fn parse_positional(
        self,
    ) -> Result<(String, Offset, Length), Box<dyn std::error::Error>> {
        // Strip out any standalone "--" delimiter passed between arguments (e.g. before negative offsets)
        // because clap trailing_var_arg includes "--" in positional values when placed after positional args.
        let positional: Vec<String> = self.positional.into_iter().filter(|s| s != "--").collect();
        if positional.len() < 3 {
            return Err("Expected FILE, OFFSET, and LENGTH arguments".into());
        }

        // Try to detect order by checking if first arg is a valid file
        // Then try offset/length parsing
        let file: String;
        let offset: Offset;
        let length: Length;

        // Strategy: check if first arg could be offset/length
        let first_is_offset = {
            if Offset::from_str(&positional[0]).is_ok() {
                if let Ok(_len) = Length::from_str(&positional[1]) {
                    true // First two look like offset/length
                } else {
                    false
                }
            } else {
                false
            }
        };

        if first_is_offset {
            // OFFSET LENGTH FILE order
            if positional.len() < 3 {
                return Err("Expected OFFSET LENGTH FILE or FILE OFFSET LENGTH".into());
            }
            offset =
                Offset::from_str(&positional[0]).map_err(|e| format!("Invalid offset: {}", e))?;
            length =
                Length::from_str(&positional[1]).map_err(|e| format!("Invalid length: {}", e))?;
            file = positional[2].clone();
        } else {
            // FILE OFFSET LENGTH order
            if positional.len() < 3 {
                return Err("Expected FILE OFFSET LENGTH or OFFSET LENGTH FILE".into());
            }
            file = positional[0].clone();
            offset =
                Offset::from_str(&positional[1]).map_err(|e| format!("Invalid offset: {}", e))?;
            length =
                Length::from_str(&positional[2]).map_err(|e| format!("Invalid length: {}", e))?;
        }

        Ok((file, offset, length))
    }
}

#[derive(Clone, ValueEnum)]
pub(crate) enum OutputFormat {
    #[clap(name = "decimal")]
    Decimal,
    #[clap(name = "hex")]
    Hex,
    #[clap(name = "binary")]
    Binary,
    #[clap(name = "ascii")]
    Ascii,
    #[clap(name = "hex-ascii")]
    HexAscii,
    #[clap(name = "interleaved", alias = "hex-ascii-interleaved")]
    Interleaved,
}
