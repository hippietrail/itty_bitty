use std::str::FromStr;

#[derive(Debug)]
pub(crate) enum OffsetError {
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

#[allow(dead_code)]
fn parse_offset(s: &str) -> Result<Offset, String> {
    Offset::from_str(s).map_err(|e| e.to_string())
}

#[allow(dead_code)]
fn parse_length(s: &str) -> Result<Length, String> {
    Length::from_str(s).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub(crate) struct Length {
    bits: u64,
}

impl Length {
    pub(crate) fn to_bits(&self) -> u64 {
        self.bits
    }
}

impl FromStr for Length {
    type Err = OffsetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace(&[',', '_', '\''][..], "");

        // Try to split into bytes:bits
        let (num_str, bits_part) = if let Some((a, b)) = s.split_once([':', '.']) {
            let bits = b
                .parse::<u64>()
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
        } else if let Some(hex_digits) = num_str.strip_prefix('$') {
            u64::from_str_radix(hex_digits, 16)
        } else if num_str.ends_with('h') || num_str.ends_with('H') {
            u64::from_str_radix(&num_str[..num_str.len() - 1], 16)
        } else {
            num_str.parse::<u64>()
        }
        .map_err(|e| OffsetError::ParseError(e.to_string()))?;

        // If bits specified, num is bytes; otherwise num is total bits
        let total_bits = if let Some(bit_offset) = bits_part {
            (num * 8) + bit_offset
        } else {
            num
        };

        Ok(Length { bits: total_bits })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Offset {
    pub(crate) bytes: u64,
    pub(crate) bits: u32, // 0-7
    pub(crate) is_negative: bool,
}

impl Offset {
    pub(crate) fn to_bits(&self) -> i64 {
        let total_bits = (self.bytes * 8) as i64 + self.bits as i64;
        if self.is_negative {
            -total_bits
        } else {
            total_bits
        }
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
        let (num_str, bits_part) = if let Some((a, b)) = s.split_once([':', '.']) {
            let bits = b
                .parse::<u32>()
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
        } else if let Some(stripped) = num_str.strip_prefix('$') {
            u64::from_str_radix(stripped, 16)
        } else if num_str.ends_with('h') || num_str.ends_with('H') {
            u64::from_str_radix(&num_str[..num_str.len() - 1], 16)
        } else {
            num_str.parse::<u64>()
        }
        .map_err(|e| OffsetError::ParseError(e.to_string()))?;

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
