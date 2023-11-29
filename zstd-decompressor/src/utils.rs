use num_traits::{self, NumCast};

/// Get 'n' lowest bits from 'value' in little-endiand notation, returns the new 'value' and the n bits
pub(crate) fn get_n_bits<T: num_traits::PrimInt + From<u8>>(value: T, n: usize) -> (T, T) {
    let and: T = (<T as NumCast>::from(1u8).unwrap() << n) - 1.into();

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

#[cfg(test)]
mod tests {
    use super::{get_n_bits, int_from_array};

    #[test]
    fn get_n_bits_ok_test() {
        let int_8: u8 = 0b10;
        let int_16: u16 = 0b10;
        let int_32: u32 = 0b10;
        let int_64: u64 = 0b10;

        assert_eq!((2, 0), get_n_bits(int_8, 2));
        assert_eq!((2, 0), get_n_bits(int_16, 2));
        assert_eq!((2, 0), get_n_bits(int_32, 2));
        assert_eq!((2, 0), get_n_bits(int_64, 2));
    }

    #[test]
    fn get_n_bits_ok_test_bis() {
        let int_8 = 0b000000011010;

        assert_eq!((0, 0b1101), get_n_bits(int_8, 1));
        assert_eq!((2, 0b110), get_n_bits(int_8, 2));
        assert_eq!((2, 0b11), get_n_bits(int_8, 3));
        assert_eq!((10, 0b1), get_n_bits(int_8, 4));
        assert_eq!((26, 0), get_n_bits(int_8, 10));
    }

    #[test]
    fn int_to_array_ok() {
        let array = [0u8, 1]; // u16 at least

        assert_eq!(256, int_from_array(&array));
    }

    #[test]
    fn int_to_array_to_bigger_ok() {
        let array = [0u8, 1]; // u16 at least

        assert_eq!(256u64, int_from_array(&array));
    }

    #[test]
    #[should_panic]
    fn int_to_array_to_smaller_int_should_panic() {
        let array = [1u8, 0]; // u16 at least

        int_from_array::<u8>(&array);
    }
}
