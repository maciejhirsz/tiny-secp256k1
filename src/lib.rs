#![no_std]

mod big_num;

use big_num::BigNum;

pub fn is_valid_secret(bytes: &[u8]) -> bool {
    if bytes.len() != 32 {
        return false;
    }

    let num = BigNum::from(bytes);

    !num.is_overflow() && !num.is_zero()
}
