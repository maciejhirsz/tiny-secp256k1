use big_num::{self, BigNum};

pub struct ECJPoint {
    pub x: BigNum,
    pub y: BigNum,
    pub z: BigNum,
    pub inf: bool
}

impl Default for ECJPoint {
    fn default() -> Self {
        ECJPoint {
            x: big_num::ZERO,
            y: big_num::ZERO,
            z: big_num::ONE,
            inf: false
        }
    }
}

impl ECJPoint {
    pub fn new(x: BigNum, y: BigNum, z: BigNum) -> Self {
        ECJPoint {
            x: x,
            y: y,
            z: z,
            inf: false
        }
    }
}
