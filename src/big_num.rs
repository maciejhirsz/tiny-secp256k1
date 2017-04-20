use core::ops::{Add, Sub, Shr};
use core::cmp::Ordering;

#[derive(Copy, Clone, Debug, Eq)]
pub struct BigNum {
	negative: bool,
	len: usize,
	words: [u32; 10]
}

impl PartialEq for BigNum {
	#[inline]
	fn eq(&self, other: &BigNum) -> bool {
		self.negative == other.negative && self.words() == other.words()
	}
}

impl Ord for BigNum {
	fn cmp(&self, other: &BigNum) -> Ordering {
		if self.len != other.len {
			return self.len.cmp(&other.len);
		}

		for i in (0..self.len).rev() {
			let ord = self.words[i].cmp(&other.words[i]);

			if ord != Ordering::Equal {
				return ord;
			}
		}

		Ordering::Equal
	}
}

impl PartialOrd for BigNum {
    fn partial_cmp(&self, other: &BigNum) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for BigNum {
	type Output = BigNum;

	fn add(mut self, mut rhs: BigNum) -> Self {
		if self.negative != rhs.negative {
			let mut res;

			if self.negative {
				self.negative = false;
				res = self - rhs;
				res.negative = !res.negative;
			} else {
				rhs.negative = false;
				res = self - rhs;
			}

			res.norm_sign();

			return res;
		}

		let (a, b) = match self.len > rhs.len {
			true => (self, rhs),
			false => (rhs, self)
		};

		let mut i = 0;
		let mut carry = 0;

		while i < b.len {
			let word = a.words[i] + b.words[i] + carry;
			self.words[i] = word & 0x03ffffff;
			carry = word >> 26;

			i += 1;
		}

		while carry != 0 && i < a.len {
			let word = a.words[i] + carry;
			self.words[i] = word & 0x03ffffff;
			carry = word >> 26;

			i += 1;
		}

		self.len = a.len;

		if carry != 0 {
			// It's fine if this panics
			self.words[self.len] = carry;
			self.len += 1;
		} else {
			self.words[i..].copy_from_slice(&a.words[i..]);
		}

		self
	}
}

impl Sub for BigNum {
	type Output = BigNum;

	fn sub(mut self, mut rhs: BigNum) -> Self {
		if self.negative != rhs.negative {
			let mut res;

			if self.negative {
				self.negative = false;
				res = self + rhs;
				res.negative = true;
			} else {
				rhs.negative = false;
				res = self + rhs;
			}

			res.norm_sign();

			return res;
		}

		if self == rhs {
			return ZERO;
		}

		let (a, b) = match self > rhs {
			true => (self, rhs),
			false => {
				self.negative = !self.negative;

				(rhs, self)
			}
		};

		let mut i = 0;
		let mut carry = 0;

		while i < b.len {
			let word = a.words[i] - b.words[i] + carry;
			carry = word >> 26;
			self.words[i] = word & 0x03ffffff;

			i += 1;
		}

		while carry != 0 && i < a.len {
			let word = a.words[i] + carry;
			carry = word >> 26;
			self.words[i] = word & 0x03ffffff;

			i += 1;
		}

		if carry == 0 && i < a.len {
			self.words[i..].copy_from_slice(&a.words[i..]);
		}

		if i > self.len {
			self.len = i;
		}

		self.strip();
		self.norm_sign();

		self
	}
}

impl Shr<u32> for BigNum {
	type Output = BigNum;

	fn shr(mut self, shift: u32) -> BigNum {
		let mask = (1 << shift) - 1;
		let m = 26 - shift;
		let mut carry = 0;

		for word in self.words_mut().iter_mut().rev() {
			let tmp = *word;
			*word = (carry << m) | (*word >> shift);
			carry = tmp & mask;
		}

		if self.len > 1 && self.words[self.len - 1] == 0 {
			self.len -= 1;
		}

		self
	}
}

#[inline]
fn read_u32(buf: &[u8], offset: isize) -> u32 {
	u32::from_be(unsafe { *(buf.as_ptr().offset(offset) as *const u32) })
}

impl<'a> From<&'a [u8]> for BigNum {
	fn from(buf: &'a [u8]) -> Self {
		assert!(buf.len() == 32);

		let mut bn = BigNum {
			negative: false,
			len: 10,
			words: [
				read_u32(buf, 28) & 0x03ffffff,
				(read_u32(buf, 25) & 0x0fffffff) >> 2,
				(read_u32(buf, 22) & 0x3fffffff) >> 4,
				read_u32(buf, 19) >> 6,

				read_u32(buf, 15) & 0x03ffffff,
				(read_u32(buf, 12) & 0x0fffffff) >> 2,
				(read_u32(buf, 9) & 0x3fffffff) >> 4,
				read_u32(buf, 6) >> 6,

				read_u32(buf, 2) & 0x03ffffff,
				(read_u32(buf, 0) & 0x00ffffff) >> 2
			]
		};

		bn.strip();

		bn
	}
}

impl From<u32> for BigNum {
	fn from(n: u32) -> Self {
		let mut words = [0; 10];

		words[0] = n & 0x03ffffff;

		BigNum {
			negative: false,
			len: 1,
			words: words
		}
	}
}

impl BigNum {
	#[inline]
	fn strip(&mut self) {
		while self.len > 1 && self.words[self.len - 1] == 0 {
			self.len -= 1;
		}
	}

	#[inline]
	fn words(&self) -> &[u32] {
		&self.words[..self.len]
	}

	#[inline]
	fn words_mut(&mut self) -> &mut [u32] {
		&mut self.words[..self.len]
	}

	#[inline]
	fn norm_sign(&mut self) {
		if self.len == 0 && self.words[0] == 0 {
			self.negative = false;
		}
	}

	#[inline]
	pub fn is_overflow(&self) -> bool {
		self >= &N
	}

	#[inline]
	pub fn is_zero(&self) -> bool {
		self.len == 1 && self.words[0] == 0
	}
}

pub const ZERO: BigNum = BigNum {
	negative: false,
	len: 1,
	words: [0; 10]
};

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
pub const N: BigNum = BigNum {
	negative: false,
	len: 10,
	words: [
		0x0364141,
		0x097a334,
		0x203bbfd,
		0x39abd22,
		0x2baaedc,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x03fffff
	]
};

// 7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0
pub const NH: BigNum = BigNum {
	negative: false,
	len: 10,
	words: [
		0x01b20a0,
		0x24bd19a,
		0x101ddfe,
		0x1cd5e91,
		0x35d576e,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x01fffff
	]
};

// 000000000000000000000000000000014551231950B75FC4402DA1732FC9BEBF
pub const NC: BigNum = BigNum {
	negative: false,
	len: 5,
	words: [
		0x3c9bebf,
		0x3685ccb,
		0x1fc4402,
		0x06542dd,
		0x1455123,
		0,
		0,
		0,
		0,
		0
	]
};

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
pub const P: BigNum = BigNum {
	negative: false,
	len: 10,
	words: [
		0x3fffc2f,
		0x3ffffbf,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x3ffffff,
		0x03fffff
	]
};

// P - N = 14551231950b75fc4402da1722fc9baee,
pub const PSN: BigNum = BigNum {
	negative: false,
	len: 5,
	words: [
		0x3c9baee,
		0x3685c8b,
		0x1fc4402,
		0x06542dd,
		0x1455123,
		0,
		0,
		0,
		0,
		0
	]
};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn produces_valid_n() {
		let bytes: &[u8] = &[
			0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFE,
			0xBA,0xAE,0xDC,0xE6,0xAF,0x48,0xA0,0x3B,0xBF,0xD2,0x5E,0x8C,0xD0,0x36,0x41,0x41,
		];

		assert_eq!(BigNum::from(bytes), N);
	}

	#[test]
	fn produces_valid_nh() {
		let nh = N >> 1;

		assert_eq!(nh, NH);
	}

	#[test]
	fn produces_valid_nc() {
		let bytes: &[u8] = &[
			0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01,
			0x45,0x51,0x23,0x19,0x50,0xB7,0x5F,0xC4,0x40,0x2D,0xA1,0x73,0x2F,0xC9,0xBE,0xBF,
		];

		assert_eq!(BigNum::from(bytes), NC);
	}

	#[test]
	fn produces_valid_p() {
		let bytes: &[u8] = &[
			0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
			0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFE,0xFF,0xFF,0xFC,0x2F,
		];

		assert_eq!(BigNum::from(bytes), P);
	}

	#[test]
	fn produces_valid_psn() {
		let psn = P - N;

		assert_eq!(psn, PSN);
	}

	#[test]
	fn is_zero() {
		assert_eq!(ZERO.is_zero(), true);
		assert_eq!(N.is_zero(), false);
	}

	#[test]
	fn is_overflow() {
		let np1 = N + BigNum::from(1);
		let ns1 = N - BigNum::from(1);

		assert_eq!(np1.is_overflow(), true);
		assert_eq!(N.is_overflow(), true);
		assert_eq!(ns1.is_overflow(), false);
	}

	#[test]
	fn n_sub_one() {
		let bn = N - BigNum::from(1);

		let expected_bytes: &[u8] = &[
			0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xfe,
			0xba,0xae,0xdc,0xe6,0xaf,0x48,0xa0,0x3b,0xbf,0xd2,0x5e,0x8c,0xd0,0x36,0x41,0x40,
		];

		assert_eq!(bn, BigNum::from(expected_bytes));
	}
}
