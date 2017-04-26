use big_num::{self, BigNum};

#[derive(Clone, Copy, Debug)]
pub struct ECPoint {
    pub x: BigNum,
    pub y: BigNum,
    pub inf: bool,
}

impl ECPoint {
    pub fn new(x: BigNum, y: BigNum) -> Self {
        ECPoint {
            x: x,
            y: y,
            inf: false
        }
    }

    pub fn inf() -> Self {
        ECPoint {
            x: big_num::ZERO,
            y: big_num::ZERO,
            inf: true
        }
    }

    pub fn to_public_key(&self) -> [u8; 65] {
        let mut public_key = [0u8; 65];

        public_key[0] = 0x04;

        self.x.write_bytes_to(&mut public_key[1..33]);
        self.y.write_bytes_to(&mut public_key[33..65]);

        return public_key;
    }

    pub fn dbl(&self) -> ECPoint {
        if self.inf {
            return *self;
        }

        let yy = self.y.red_add(self.y);

        if yy == 0 {
            return ECPoint::inf();
        }

        let xsqr = self.x.red_sqr();
        let s = xsqr.red_add(xsqr).red_add(xsqr).red_mul(yy.red_invm());

        let nx = s.red_sqr().red_sub(self.x.red_add(self.x));
        let ny = s.red_mul(self.x.red_sub(nx)).red_sub(self.y);

        ECPoint::new(nx, ny)
    }
}
