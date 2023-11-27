use num_traits::{self, NumCast};

/// Get 'n' lowest bits from 'value' in little-endiand notation, returns the new 'value' and the n bits
pub(crate) fn get_n_bits<T: num_traits::PrimInt + From<u8>>(value: T, n: usize) -> (T, T) {
    let mut and: T = 1.into();
    for _ in 0..n {
        and = (and << 1) | 1.into();
    }

    let res = value & and;
    let new_value = value >> n;

    (res, new_value)
}

/// Creates an int from a u8 array in little-endian form
pub(crate) fn int_from_array<T: num_traits::PrimInt + From<u8>>(input: &[u8]) -> T {
    let mut res: T = 0.into();
    input
        .iter()
        .enumerate()
        .for_each(|(i, v)| res = res | (<T as NumCast>::from(*v).unwrap() << (8 * i)));

    res
}
