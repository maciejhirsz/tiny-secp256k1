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

impl<'a> Add<&'a ECPoint> for ECPoint {
    type Output = ECPoint;

    fn add(mut self, rhs: &'a ECPoint) -> ECPoint {
        self.add_assign(rhs);
        self
    }
}

impl<'a> AddAssign<&'a ECPoint> for ECPoint {
    fn add_assign(&mut self, rhs: &ECPoint) {
        // O + P = P
        if self.inf {
            *self = *rhs;
            return;
        }

        // P + O = P
        if rhs.inf {
            return;
        }

        if self.x == rhs.x {
            // P + P = 2P
            if self.y == rhs.y {
                self.double();
                return;
            }
            // P + (-P) = O
            self.inf = true;
            return;
        }

        // s = (y - yp) / (x - xp)
        // nx = s**2 - x - xp
        // ny = s * (x - nx) - y
        let mut s = self.y.red_sub(&rhs.y);

        if s != 0 {
            s.red_mul_mut(&self.x.red_sub(&rhs.x).red_invm())
        }

        let nx = s.red_sqr().red_sub(&self.x).red_sub(&rhs.x);
        self.y = s.red_mul(&self.x.red_sub(&nx)).red_sub(&self.y);
        self.x = nx;
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

    pub fn double(&mut self) {
        if self.inf {
            return;
        }

        let yy = self.y.red_add(&self.y);

        if yy == 0 {
            self.inf = true;
            return;
        }

        let xsqr = self.x.red_sqr();
        let s = xsqr.red_add(&xsqr).red_add(&xsqr).red_mul(&yy.red_invm());

        let nx = s.red_sqr().red_sub(&self.x.red_add(&self.x));
        self.y = s.red_mul(&self.x.red_sub(&nx)).red_sub(&self.y);
        self.x = nx;
    }


    pub fn neg(&self) -> ECPoint {
        if self.inf {
            *self
        } else {
            ECPoint::new(self.x, self.y.red_neg())
        }
    }
}
