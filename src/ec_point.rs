use big_num::{self, BigNum};
use core::ops::{Add, AddAssign}; //, Sub, SubAssign, Mul, MulAssign, Shr, ShrAssign};

#[derive(Clone, Copy, Debug)]
pub struct ECPoint {
    pub x: BigNum,
    pub y: BigNum,
    pub inf: bool
}

pub const INF: ECPoint = ECPoint {
    x: big_num::ZERO,
    y: big_num::ZERO,
    inf: true
};

impl Add for ECPoint {
    type Output = ECPoint;

    fn add(self, rhs: ECPoint) -> ECPoint {
        // O + P = P
        if self.inf {
            return rhs;
        }

        // P + O = P
        if rhs.inf {
            return self;
        }

        if self.x == rhs.x {
            // P + P = 2P
            if self.y == rhs.y {
                return self.dbl();
            }
            // P + (-P) = O
            return INF;
        }

        // s = (y - yp) / (x - xp)
        // nx = s**2 - x - xp
        // ny = s * (x - nx) - y
        let mut s = self.y.red_sub(rhs.y);

        if s != 0 {
            s = s.red_mul(self.x.red_sub(rhs.x).red_invm())
        }

        let nx = s.red_sqr().red_sub(self.x).red_sub(rhs.x);
        let ny = s.red_mul(self.x.red_sub(nx)).red_sub(self.y);

        return ECPoint::new(nx, ny);
    }
}

impl AddAssign for ECPoint {
    fn add_assign(&mut self, rhs: ECPoint) {
        *self = *self + rhs;
    }
}

impl ECPoint {
    pub fn new(x: BigNum, y: BigNum) -> Self {
        ECPoint {
            x: x,
            y: y,
            inf: false
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
            return INF;
        }

        let xsqr = self.x.red_sqr();
        let s = xsqr.red_add(xsqr).red_add(xsqr).red_mul(yy.red_invm());

        let nx = s.red_sqr().red_sub(self.x.red_add(self.x));
        let ny = s.red_mul(self.x.red_sub(nx)).red_sub(self.y);

        ECPoint::new(nx, ny)
    }

    #[inline]
    pub fn neg(&self) -> ECPoint {
        if self.inf {
            *self
        } else {
            ECPoint::new(self.x, self.y.red_neg())
        }
    }
}
