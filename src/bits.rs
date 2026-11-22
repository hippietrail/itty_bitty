use bitvec::{
    order::{Lsb0, Msb0},
    slice::BitSlice,
};
use num_bigint::BigUint;

pub(crate) fn extract_bits_to_biguint(bits: &BitSlice<u8, Msb0>) -> BigUint {
    let n = bits.len();
    if n == 0 {
        return BigUint::ZERO;
    }

    let num_bytes = n.div_ceil(8);
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

pub(crate) fn extract_bits_to_biguint_lsb(bits: &BitSlice<u8, Lsb0>) -> BigUint {
    let mut result = BigUint::ZERO;
    for (i, bit) in bits.iter().enumerate() {
        if *bit {
            result |= BigUint::from(1u8) << i;
        }
    }
    result
}
