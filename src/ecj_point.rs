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
		let ax = val.x.red_mul(&zinv2);
		let mut ay = val.y.red_mul(&zinv2);
		ay.red_mul_mut(&zinv);

		ECPoint::new(ax, ay)
	}
}

impl<'a> Add<&'a ECJPoint> for ECJPoint {
	type Output = ECJPoint;

	#[inline]
	fn add(mut self, p: &ECJPoint) -> ECJPoint {
		self.add_assign(p);
		self
	}
}

impl<'a> AddAssign<&'a ECJPoint> for ECJPoint {
	fn add_assign(&mut self, p: &ECJPoint) {
		// O + P = P
		if self.inf() {
			*self = *p;
			return;
		}

		// P + O = P;
		if p.inf() {
			return;
		}

	  	// http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-add-1998-cmo-2
  		// 12M + 4S + 7A
  		let pz2 = p.z.red_sqr();
  		let z2 = self.z.red_sqr();
  		let u1 = self.x.red_mul(&pz2);
  		let u2 = p.x.red_mul(&z2);
  		let s1 = self.y.red_mul(&pz2).red_mul(&p.z);
  		let s2 = p.y.red_mul(&z2).red_mul(&self.z);

  		let h = u1.red_sub(&u2);
  		let r = s1.red_sub(&s2);

  		if h == 0 {
  			if r == 0 {
  				self.double();
  				return;
  			}

  			*self = ECJPoint::default();
  			return;
  		}

  		let h2 = h.red_sqr();
  		let v = u1.red_mul(&h2);
  		let h3 = h2.red_mul(&h);

  		self.x = r.red_sqr().red_add(&h3).red_sub(&v).red_sub(&v);
  		self.y = r.red_mul(&v.red_sub(&self.x)).red_sub(&s1.red_mul(&h3));
  		self.z = self.z.red_mul(&p.z).red_mul(&h);
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

	pub fn mixed_add(&mut self, p: &ECPoint) {
		// O + P = P
		if self.inf() {
			*self = p.clone().into();
			return;
		}

		// P + O = P
		if p.inf {
			return;
		}

		// http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-add-1998-cmo-2
		//   with p.z = 1
		// 8M + 3S + 7A
		let z2 = self.z.red_sqr();
		let u2 = p.x.red_mul(&z2);
		let mut s2 = p.y.red_mul(&z2);
		s2.red_mul_mut(&self.z);

		let h = self.x.red_sub(&u2);
		let r = self.y.red_sub(&s2);

		if h == 0 {
			if r == 0 {
				self.double();
				return;
			}
			self.x = big_num::ONE;
			self.y = big_num::ONE;
			self.z = big_num::ZERO;
			return;
		}

		let h2 = h.red_sqr();
		let v = self.x.red_mul(&h2);
		let h3 = h2.red_mul(&h);

		self.x = r.red_sqr().red_add(&h3).red_sub_twice(&v);
		self.y = r.red_mul(&v.red_sub(&self.x)).red_sub(&self.y.red_mul(&h3));
		self.z.red_mul_mut(&h);
	}

	pub fn double(&mut self) {
		if self.inf() {
			return;
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
		    let mut s = self.x.red_add(&yy).red_sqr().red_sub(&xx).red_sub(&yyyy);
		    s.red_double();
		    let m = xx.red_add(&xx).red_add(&xx);
		    let t = m.red_sqr().red_sub(&s).red_sub(&s);

		    let mut yyyy8 = yyyy;
		    yyyy8.red_double(); // x2
		    yyyy8.red_double(); // x4
		    yyyy8.red_double(); // x8

		    nx = t;
		    ny = m.red_mul(&s.red_sub(&t)).red_sub(&yyyy8);
		    nz = self.y.red_add(&self.y);
		} else {
		    // http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
    		// 2M + 5S + 6A + 3*2 + 1*3 + 1*8
    		let a = self.x.red_sqr();
    		let b = self.y.red_sqr();
    		let c = b.red_sqr();
    		let mut d = self.x.red_add(&b).red_sqr().red_sub(&a).red_sub(&c);
    		d.red_double();
    		let e = a.red_add(&a).red_add(&a);
    		let f = e.red_sqr();

    		let mut c8 = c;
    		c8.red_double(); // x2
    		c8.red_double(); // x4
    		c8.red_double(); // x8

    		nx = f.red_sub(&d).red_sub(&d);
    		ny = e.red_mul(&d.red_sub(&nx)).red_sub(&c8);
    		nz = self.y.red_mul(&self.z);
    		nz = nz.red_add(&nz);
		}

		self.x = nx;
		self.y = ny;
		self.z = nz;
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
		let mut ecj = ECJPoint::default();

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
		ecj.mixed_add(&ecpoint);
		assert_eq!(ecj, expected);
	}

	#[test]
	fn ecj_point_add() {
		let xa: &[u8] = &[
			0x76,0xc4,0xb8,0xb7,0x60,0x3d,0x29,0xde,0x9e,0x97,0x50,0x2f,0xbc,
			0x50,0xef,0x65,0xe0,0xfe,0x61,0x33,0x09,0x5b,0x98,0x00,0x56,0x1c,
			0x70,0xf7,0xb8,0x10,0x89,0x9a
		];
		let ya: &[u8] = &[
			0x4c,0xd4,0x83,0x89,0x82,0x23,0x24,0xe8,0x0a,0x0f,0x2c,0x65,0x49,
			0xef,0x7c,0x57,0x89,0x99,0x59,0x7c,0x1d,0xaa,0xad,0xc7,0x23,0xb9,
			0x89,0x3a,0xc6,0x12,0x15,0x64
		];
		let za: &[u8] = &[
			0x96,0xd3,0x5d,0xf3,0xe7,0xf7,0x2e,0x24,0xed,0xf5,0xf2,0x32,0xbc,
			0x3f,0xd7,0xc5,0x3d,0xe3,0xff,0xc2,0x07,0x5a,0xf6,0x28,0x15,0xd6,
			0xb2,0x8a,0xc7,0x09,0xa3,0xb7
		];

		let xb: &[u8] = &[
			0x28,0x51,0xe5,0x2e,0xcf,0x11,0x7d,0x69,0x48,0x2a,0xd9,0x8b,0x4a,
			0x87,0xae,0xcb,0x5a,0xb1,0x8a,0x03,0x55,0x1a,0x41,0x4a,0x04,0x61,
			0x2f,0x6e,0x8f,0xdb,0x16,0x89
		];
		let yb: &[u8] = &[
			0xf8,0xa8,0x3e,0x40,0x03,0x8c,0x34,0xbd,0x24,0xac,0x40,0x90,0xb1,
			0x80,0x4e,0x6b,0xec,0xb6,0xe1,0xa2,0x1d,0x48,0x07,0xe0,0xa1,0xef,
			0x87,0xdf,0x43,0x45,0xa9,0x94
		];
		let zb: &[u8] = &[
			0xe9,0x07,0xe7,0x10,0x61,0x2c,0xd9,0x65,0x7e,0x2d,0x98,0x15,0xe0,
			0x67,0x79,0xb7,0x16,0xa1,0x93,0x11,0x4d,0xd5,0xdc,0x22,0x57,0x76,
			0xab,0x4f,0x0f,0x21,0x52,0xbb
		];

		let xr: &[u8] = &[
			0xfc,0x40,0xaf,0xac,0x87,0x2e,0xe1,0xb6,0xdc,0xe0,0x55,0xf3,0x38,
			0x5a,0x9f,0x59,0x45,0xce,0x13,0x65,0xf8,0x38,0x5a,0x42,0x91,0xaf,
			0xb1,0xd7,0x2a,0x87,0xf7,0x45
		];
		let yr: &[u8] = &[
			0x25,0xd4,0xb2,0x42,0x72,0x1f,0x44,0x85,0x6d,0xd2,0x36,0xa4,0x04,
			0x7b,0x40,0xa3,0xe1,0xe7,0xdb,0x9b,0x81,0x74,0xe9,0x38,0x29,0x82,
			0x80,0x53,0x39,0xa1,0xbe,0x31
		];
		let zr: &[u8] = &[
			0x4f,0xef,0x8b,0xb3,0x1c,0x65,0x91,0x53,0x55,0x81,0x74,0xb1,0x11,
			0x17,0xec,0x55,0x0d,0xf3,0xef,0x73,0xfc,0x12,0x08,0xd7,0xe6,0xe3,
			0xb8,0x98,0xe4,0x85,0x9c,0x5b
		];

		let a = ECJPoint::new(xa.into(), ya.into(), za.into());
		let b = ECJPoint::new(xb.into(), yb.into(), zb.into());
		let r = ECJPoint::new(xr.into(), yr.into(), zr.into());

		assert_eq!(a + &b, r);
	}
}
