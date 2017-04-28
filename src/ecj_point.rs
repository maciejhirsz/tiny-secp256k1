use core::ops::{Add, AddAssign};
use big_num::{self, BigNum};
use ec_point::{self, ECPoint};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ECJPoint {
	pub x: BigNum,
	pub y: BigNum,
	pub z: BigNum
}

impl Default for ECJPoint {
	#[inline]
	fn default() -> Self {
		ECJPoint {
			x: big_num::ONE,
			y: big_num::ONE,
			z: big_num::ZERO
		}
	}
}

impl From<ECPoint> for ECJPoint {
	#[inline]
	fn from(val: ECPoint) -> ECJPoint {
		if val.inf {
			return ECJPoint::default();
		}

		ECJPoint::new(val.x, val.y, big_num::ONE)
	}
}

impl From<ECJPoint> for ECPoint {
	fn from(val: ECJPoint) -> ECPoint {
		if val.inf() {
			return ec_point::INF;
		}

		let zinv = val.z.red_invm();
		let zinv2 = zinv.red_sqr();
		let ax = val.x.red_mul(zinv2);
		let ay = val.y.red_mul(zinv2).red_mul(zinv);

		ECPoint::new(ax, ay)
	}
}

impl Add for ECJPoint {
	type Output = ECJPoint;

	fn add(self, p: ECJPoint) -> ECJPoint {
		// O + P = P
		if self.inf() {
			return p;
		}

		// P + O = P;
		if p.inf() {
			return self;
		}

	  	// http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-add-1998-cmo-2
  		// 12M + 4S + 7A
  		let pz2 = p.z.red_sqr();
  		let z2 = self.z.red_sqr();
  		let u1 = self.x.red_mul(pz2);
  		let u2 = p.x.red_mul(z2);
  		let s1 = self.y.red_mul(pz2).red_mul(p.z);
  		let s2 = p.y.red_mul(z2).red_mul(self.z);

  		let h = u1.red_sub(u2);
  		let r = s1.red_sub(s2);

  		if h == 0 {
  			if r == 0 {
  				return self.dbl();
  			}

  			return ECJPoint::default();
  		}

  		let h2 = h.red_sqr();
  		let v = u1.red_mul(h2);
  		let h3 = h2.red_add(h);

  		let nx = r.red_sqr().red_add(h3).red_sub(v).red_sub(v);
  		let ny = r.red_mul(v.red_sub(nx)).red_sub(s1.red_mul(h3));
  		let nz = self.z.red_mul(p.z).red_mul(h);

  		ECJPoint::new(nx, ny, nz)
	}
}

impl AddAssign for ECJPoint {
	fn add_assign(&mut self, rhs: ECJPoint) {
		*self = *self + rhs;
	}
}

impl ECJPoint {
	#[inline]
	pub fn new(x: BigNum, y: BigNum, z: BigNum) -> Self {
		ECJPoint {
			x: x,
			y: y,
			z: z,
		}
	}

	pub fn mixed_add(&self, p: ECPoint) -> ECJPoint {
		// O + P = P
		if self.inf() {
			return p.into();
		}

		// P + O = P
		if p.inf {
			return *self;
		}

		// http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-add-1998-cmo-2
		//   with p.z = 1
		// 8M + 3S + 7A
		let z2 = self.z.red_sqr();
		let u1 = self.x;
		let u2 = p.x.red_mul(z2);
		let s1 = self.y;
		let s2 = p.y.red_mul(z2).red_mul(self.z);

		let h = u1.red_sub(u2);
		let r = s1.red_sub(s2);

		if h == 0 {
			if r == 0 {
				return self.dbl();
			}
			return ECJPoint::default();
		}

		let h2 = h.red_sqr();
		let v = u1.red_mul(h2);
		let h3 = h2.red_mul(h);

		let nx = r.red_sqr().red_add(h3).red_sub(v).red_sub(v);
		let ny = r.red_mul(v.red_sub(nx)).red_sub(s1.red_mul(h3));
		let nz = self.z.red_mul(h);

		ECJPoint::new(nx, ny, nz)
	}

	pub fn dbl(&self) -> ECJPoint {
		if self.inf() {
			return *self;
		}

		let nx;
		let ny;
		let mut nz;

		if self.z == 1 {
		    // http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-mdbl-2007-bl
		    // 1M + 5S + 6A + 3*2 + 1*3 + 1*8
		    let xx = self.x.red_sqr();
		    let yy = self.y.red_sqr();
		    let yyyy = yy.red_sqr();
		    let mut s = self.x.red_add(yy).red_sqr().red_sub(xx).red_sub(yyyy);
		    s = s.red_add(s);
		    let m = xx.red_add(xx).red_add(xx);
		    let t = m.red_sqr().red_sub(s).red_sub(s);

		    let mut yyyy8 = yyyy.red_add(yyyy); // x2
		    yyyy8 = yyyy8.red_add(yyyy8); // x4
		    yyyy8 = yyyy8.red_add(yyyy8); // x8

		    nx = t;
		    ny = m.red_mul(s.red_sub(t)).red_sub(yyyy8);
		    nz = self.y.red_add(self.y);
		} else {
		    // http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
    		// 2M + 5S + 6A + 3*2 + 1*3 + 1*8
    		let a = self.x.red_sqr();
    		let b = self.y.red_sqr();
    		let c = b.red_sqr();
    		let mut d = self.x.red_add(b).red_sqr().red_sub(a).red_sub(c);
    		d = d.red_add(d);
    		let e = a.red_add(a).red_add(a);
    		let f = e.red_sqr();

    		let mut c8 = c.red_add(c); // x2
    		c8 = c8.red_add(c8); // x4d.red
    		c8 = c8.red_add(c8); // x8

    		nx = f.red_sub(d).red_sub(d);
    		ny = e.red_mul(d.red_sub(nx)).red_sub(c8);
    		nz = self.y.red_mul(self.z);
    		nz = nz.red_add(nz);
		}

		ECJPoint::new(nx, ny, nz)
	}

	#[inline]
	pub fn inf(&self) -> bool {
		self.z == 0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ecj_point_mixed_add() {
		let ecj = ECJPoint::default();

		let x: &[u8] = &[
			0x79,0xbe,0x66,0x7e,0xf9,0xdc,0xbb,0xac,0x55,0xa0,0x62,0x95,0xce,
			0x87,0x0b,0x07,0x02,0x9b,0xfc,0xdb,0x2d,0xce,0x28,0xd9,0x59,0xf2,
			0x81,0x5b,0x16,0xf8,0x17,0x98
		];
		let y: &[u8] = &[
			0x48,0x3a,0xda,0x77,0x26,0xa3,0xc4,0x65,0x5d,0xa4,0xfb,0xfc,0x0e,
			0x11,0x08,0xa8,0xfd,0x17,0xb4,0x48,0xa6,0x85,0x54,0x19,0x9c,0x47,
			0xd0,0x8f,0xfb,0x10,0xd4,0xb8
		];
		let ecpoint = ECPoint::new(x.into(), y.into());

		let expected = ECJPoint::new(x.into(), y.into(), 1u32.into());

		assert_eq!(ecj.inf(), true);
		assert_eq!(ecj.mixed_add(ecpoint), expected);
	}
}
