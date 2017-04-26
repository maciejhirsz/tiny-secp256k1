#![no_std]

mod big_num;
mod naf;
mod ec_point;
mod ec_point_g;

use big_num::BigNum;

pub fn is_valid_secret(bytes: &[u8]) -> bool {
    if bytes.len() != 32 {
        return false;
    }

    let num = BigNum::from(bytes);

    !num.is_overflow() && num != 0
}
