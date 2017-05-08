use core::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Shr, ShrAssign};
use core::fmt::{self, Debug};
use core::cmp::Ordering;
use core::{str, mem};
use naf::{NAF, NAFRepr};

#[derive(Copy, Clone, Eq)]
pub struct BigNum {
	negative: bool,
	len: usize,
	words: [u32; 16]
}

impl Debug for BigNum {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self == 0 {
			return f.write_str("0");
		}

		let digits = b"0123456789abcdef";
		let mut buf = [b' '; 131];
		let mut i = buf.len();

		let mut n = *self;

		while n != 0 {
			i -= 1;
			buf[i] = digits[n.words[0] as usize & 0x0F];
			n >>= 4;
		}

		i -= 1;
		buf[i] = b'x';
		i -= 1;
		buf[i] = b'0';

		if n.negative {
			i -= 1;
			buf[i] = b'-';
		}

		f.write_str(
			str::from_utf8(&buf[i..]).expect("contains only ASCII hex digits; qed")
		)
	}
}

impl PartialEq for BigNum {
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

impl PartialEq<u32> for BigNum {
	fn eq(&self, other: &u32) -> bool {
		self.len == 1 && self.words[0] == *other
	}
}

impl<'a> PartialEq<u32> for &'a BigNum {
	fn eq(&self, other: &u32) -> bool {
		self.len == 1 && self.words[0] == *other
	}
}

impl PartialOrd<u32> for BigNum {
	fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
		if self.len > 1 {
			return Some(Ordering::Greater);
		}

		Some(self.words[0].cmp(other))
	}
}

impl<'a> Add<&'a BigNum> for BigNum {
	type Output = BigNum;

	fn add(mut self, rhs: &BigNum) -> Self {
		self.add_assign(rhs);
		self
	}
}

impl<'a> AddAssign<&'a BigNum> for BigNum {
	fn add_assign(&mut self, rhs: &BigNum) {
		if self.negative != rhs.negative {
			if self.negative {
				self.negative = false;
				self.sub_assign(rhs);
				self.negative = !self.negative;
			} else {
				let mut flipped = *rhs;
				flipped.negative = false;
				self.sub_assign(&flipped);
			}

			self.norm_sign();

			return;
		}

		if rhs.len > self.len {
			for i in self.len..rhs.len {
				self.words[i] = 0;
			}

			self.len = rhs.len;
		}

		let mut i = 0;
		let mut carry = 0u64;

		while i < rhs.len {
			let word = self.words[i] as u64 + rhs.words[i] as u64 + carry;
			self.words[i] = word as u32;
			carry = word >> 32;

			i += 1;
		}

		while carry != 0 && i < self.len {
			let word = self.words[i] as u64 + carry;
			self.words[i] = word as u32;
			carry = word >> 32;

			i += 1;
		}

		if carry != 0 {
			self.words[self.len] = carry as u32;
			self.len += 1;
		}
	}
}

impl Add<u32> for BigNum {
	type Output = BigNum;

	#[inline]
	fn add(mut self, rhs: u32) -> Self {
		self += rhs;
		self
	}
}

impl AddAssign<u32> for BigNum {
	fn add_assign(&mut self, rhs: u32) {
		let mut carry = rhs as u64;

		for word in self.words_mut() {
			carry += *word as u64;
			*word = carry as u32;
			carry >>= 32;

			if carry == 0 {
				return;
			}
		}

		// overflowing, need to add a new word
		self.words[self.len] = carry as u32;
		self.len += 1;
	}
}

impl<'a> Sub<&'a BigNum> for BigNum {
	type Output = BigNum;

	#[inline]
	fn sub(mut self, rhs: &BigNum) -> Self {
		self.sub_assign(rhs);
		self
	}
}

impl<'a> SubAssign<&'a BigNum> for BigNum {
	fn sub_assign(&mut self, rhs: &BigNum) {
		if self.negative != rhs.negative {
			if self.negative {
				self.negative = false;
				self.add_assign(rhs);
				self.negative = true;
			} else {
				let mut flipped = *rhs;
				flipped.negative = false;
				self.add_assign(&flipped);
			}

			self.norm_sign();

			return;
		}

		if &*self == rhs {
			self.len = 1;
			self.negative = false;
			self.words[0] = 0;

			return;
		}

		if rhs > self {
			let tmp = *self;
			*self = *rhs;
			self.sub_assign(&tmp);
			self.negative = true;
			return;
		}

		let mut i = 0;
		let mut carry = 0i64;

		while i < rhs.len {
			let word = self.words[i] as i64 - rhs.words[i] as i64 + carry;
			carry = word >> 32;
			self.words[i] = word as u32;

			i += 1;
		}

		while carry != 0 && i < self.len {
			let word = self.words[i] as i64 + carry;
			carry = word >> 32;
			self.words[i] = word as u32;

			i += 1;
		}

		if i > self.len {
			self.len = i;
		}

		self.strip();
		self.norm_sign();
	}
}

impl<'a> Mul<&'a BigNum> for BigNum {
	type Output = BigNum;

	#[inline]
	fn mul(mut self, rhs: &BigNum) -> BigNum {
		self.mul_assign(rhs);
		self
	}
}

impl<'a> MulAssign<&'a BigNum> for BigNum {
	fn mul_assign(&mut self, rhs: &BigNum) {
		if self.len == 8 && rhs.len == 8 {
			self.mul8x8(rhs);
			return;
		}

		if rhs.len == 1 {
			self.mul(rhs.words[0]);
			return;
		}

		let mut res = BigNum {
			negative: false,
			words: [0; 16],
			len: self.len + rhs.len - 1
		};

		let mut carry = self.words[0] as u64 * rhs.words[0] as u64;
		res.words[0] = carry as u32;

		carry >>= 32;

		for k in 1..res.len {
			let mut ncarry = carry >> 32;
			let mut rword = carry as u32;

			let j_low = if k > self.len { k - self.len + 1 } else { 0 };
			let j_high = if rhs.len > k { k + 1 } else { rhs.len };

			for j in j_low..j_high {
				let i = k - j;
				let a = self.words[i] as u64;
				let b = rhs.words[j] as u64;
				let r = a * b + rword as u64;
				ncarry += r >> 32;
				rword = r as u32;
			}

			res.words[k] = rword;
			carry = ncarry;
		}

		if carry != 0 {
			res.words[res.len] = carry as u32;
			res.len += 1;
		}

		res.strip();
		*self = res;
	}
}

impl Mul<u32> for BigNum {
	type Output = BigNum;

	#[inline]
	fn mul(mut self, rhs: u32) -> BigNum {
		self *= rhs;
		self
	}
}

impl MulAssign<u32> for BigNum {
	fn mul_assign(&mut self, rhs: u32) {
		let mut mul_carry = 0;
		let mut carry = 0;

		for word in self.words_mut() {
			let tmp = *word as u64 * rhs as u64;
			mul_carry = tmp >> 32;
			carry += (tmp as u32) as u64;
			*word = carry as u32;
			carry = (carry >> 32) + mul_carry;
		}

		if carry != 0 {
			self.words[self.len] = carry as u32;
			self.len += 1;
		}
	}
}

impl Shr<u32> for BigNum {
	type Output = BigNum;

	#[inline]
	fn shr(mut self, shift: u32) -> BigNum {
		self >>= shift;
		self
	}
}

impl ShrAssign<u32> for BigNum {
	fn shr_assign(&mut self, shift: u32) {
		let mut carry = 0;
		let m = 32 - shift;

		for word in self.words_mut().iter_mut().rev() {
			let tmp = *word as u64;
			*word = (carry | (tmp >> shift)) as u32;
			carry = tmp << m;
		}

		if self.len > 1 && self.words[self.len - 1] == 0 {
			self.len -= 1;
		}
	}
}

#[inline]
fn read_u32(buf: &[u8]) -> u32 {
	assert!(buf.len() == 4);

	// Note: while this will work on all archs, WASM is _always_ little-endian
	u32::from_be(unsafe { *(buf.as_ptr() as *const u32) })
}

#[inline]
fn write_u32(val: u32, buf: &mut [u8]) {
	assert!(buf.len() == 4);

	unsafe {
		let ptr = buf.as_mut_ptr() as *mut u32;
		*ptr = u32::to_be(val);
	}
}

impl<'a> From<&'a [u8]> for BigNum {
	fn from(buf: &'a [u8]) -> Self {
		let mut bn = BigNum {
			negative: false,
			len: buf.len() / 4,
			words: [
				read_u32(&buf[28..32]),
				read_u32(&buf[24..28]),
				read_u32(&buf[20..24]),
				read_u32(&buf[16..20]),
				read_u32(&buf[12..16]),
				read_u32(&buf[8..12]),
				read_u32(&buf[4..8]),
				read_u32(&buf[0..4]),
				0, 0, 0, 0, 0, 0, 0, 0
			]
		};

		bn.strip();

		bn
	}
}

impl From<u32> for BigNum {
	fn from(n: u32) -> Self {
		BigNum {
			negative: false,
			len: 1,
			words: [
				n, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0
			]
		}
	}
}

impl BigNum {
	fn mul8x8(&mut self, rhs: &BigNum) {
		let mut c = 0;
		let mut lo;
		let mut mid;
		let mut hi;
		let mut word;
		let al0 = self.words[0] & 0xffff;
		let ah0 = self.words[0] >> 16;
		let al1 = self.words[1] & 0xffff;
		let ah1 = self.words[1] >> 16;
		let al2 = self.words[2] & 0xffff;
		let ah2 = self.words[2] >> 16;
		let al3 = self.words[3] & 0xffff;
		let ah3 = self.words[3] >> 16;
		let al4 = self.words[4] & 0xffff;
		let ah4 = self.words[4] >> 16;
		let al5 = self.words[5] & 0xffff;
		let ah5 = self.words[5] >> 16;
		let al6 = self.words[6] & 0xffff;
		let ah6 = self.words[6] >> 16;
		let al7 = self.words[7] & 0xffff;
		let ah7 = self.words[7] >> 16;
		let bl0 = rhs.words[0] & 0xffff;
		let bh0 = rhs.words[0] >> 16;
		let bl1 = rhs.words[1] & 0xffff;
		let bh1 = rhs.words[1] >> 16;
		let bl2 = rhs.words[2] & 0xffff;
		let bh2 = rhs.words[2] >> 16;
		let bl3 = rhs.words[3] & 0xffff;
		let bh3 = rhs.words[3] >> 16;
		let bl4 = rhs.words[4] & 0xffff;
		let bh4 = rhs.words[4] >> 16;
		let bl5 = rhs.words[5] & 0xffff;
		let bh5 = rhs.words[5] >> 16;
		let bl6 = rhs.words[6] & 0xffff;
		let bh6 = rhs.words[6] >> 16;
		let bl7 = rhs.words[7] & 0xffff;
		let bh7 = rhs.words[7] >> 16;

		// k = 0
		lo = (al0 * bl0) as u64;
		mid = (al0 * bh0) as u64;
		mid += (ah0 * bl0) as u64;
		hi = (ah0 * bh0) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[0] = word as u32;
		// k = 1
		lo = (al1 * bl0) as u64;
		mid = (al1 * bh0) as u64;
		mid += (ah1 * bl0) as u64;
		hi = (ah1 * bh0) as u64;
		lo += (al0 * bl1) as u64;
		mid += (al0 * bh1) as u64;
		mid += (ah0 * bl1) as u64;
		hi += (ah0 * bh1) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[1] = word as u32;
		// k = 2
		lo = (al2 * bl0) as u64;
		mid = (al2 * bh0) as u64;
		mid += (ah2 * bl0) as u64;
		hi = (ah2 * bh0) as u64;
		lo += (al1 * bl1) as u64;
		mid += (al1 * bh1) as u64;
		mid += (ah1 * bl1) as u64;
		hi += (ah1 * bh1) as u64;
		lo += (al0 * bl2) as u64;
		mid += (al0 * bh2) as u64;
		mid += (ah0 * bl2) as u64;
		hi += (ah0 * bh2) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[2] = word as u32;
		// k = 3
		lo = (al3 * bl0) as u64;
		mid = (al3 * bh0) as u64;
		mid += (ah3 * bl0) as u64;
		hi = (ah3 * bh0) as u64;
		lo += (al2 * bl1) as u64;
		mid += (al2 * bh1) as u64;
		mid += (ah2 * bl1) as u64;
		hi += (ah2 * bh1) as u64;
		lo += (al1 * bl2) as u64;
		mid += (al1 * bh2) as u64;
		mid += (ah1 * bl2) as u64;
		hi += (ah1 * bh2) as u64;
		lo += (al0 * bl3) as u64;
		mid += (al0 * bh3) as u64;
		mid += (ah0 * bl3) as u64;
		hi += (ah0 * bh3) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[3] = word as u32;
		// k = 4
		lo = (al4 * bl0) as u64;
		mid = (al4 * bh0) as u64;
		mid += (ah4 * bl0) as u64;
		hi = (ah4 * bh0) as u64;
		lo += (al3 * bl1) as u64;
		mid += (al3 * bh1) as u64;
		mid += (ah3 * bl1) as u64;
		hi += (ah3 * bh1) as u64;
		lo += (al2 * bl2) as u64;
		mid += (al2 * bh2) as u64;
		mid += (ah2 * bl2) as u64;
		hi += (ah2 * bh2) as u64;
		lo += (al1 * bl3) as u64;
		mid += (al1 * bh3) as u64;
		mid += (ah1 * bl3) as u64;
		hi += (ah1 * bh3) as u64;
		lo += (al0 * bl4) as u64;
		mid += (al0 * bh4) as u64;
		mid += (ah0 * bl4) as u64;
		hi += (ah0 * bh4) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[4] = word as u32;
		// k = 5
		lo = (al5 * bl0) as u64;
		mid = (al5 * bh0) as u64;
		mid += (ah5 * bl0) as u64;
		hi = (ah5 * bh0) as u64;
		lo += (al4 * bl1) as u64;
		mid += (al4 * bh1) as u64;
		mid += (ah4 * bl1) as u64;
		hi += (ah4 * bh1) as u64;
		lo += (al3 * bl2) as u64;
		mid += (al3 * bh2) as u64;
		mid += (ah3 * bl2) as u64;
		hi += (ah3 * bh2) as u64;
		lo += (al2 * bl3) as u64;
		mid += (al2 * bh3) as u64;
		mid += (ah2 * bl3) as u64;
		hi += (ah2 * bh3) as u64;
		lo += (al1 * bl4) as u64;
		mid += (al1 * bh4) as u64;
		mid += (ah1 * bl4) as u64;
		hi += (ah1 * bh4) as u64;
		lo += (al0 * bl5) as u64;
		mid += (al0 * bh5) as u64;
		mid += (ah0 * bl5) as u64;
		hi += (ah0 * bh5) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[5] = word as u32;
		// k = 6
		lo = (al6 * bl0) as u64;
		mid = (al6 * bh0) as u64;
		mid += (ah6 * bl0) as u64;
		hi = (ah6 * bh0) as u64;
		lo += (al5 * bl1) as u64;
		mid += (al5 * bh1) as u64;
		mid += (ah5 * bl1) as u64;
		hi += (ah5 * bh1) as u64;
		lo += (al4 * bl2) as u64;
		mid += (al4 * bh2) as u64;
		mid += (ah4 * bl2) as u64;
		hi += (ah4 * bh2) as u64;
		lo += (al3 * bl3) as u64;
		mid += (al3 * bh3) as u64;
		mid += (ah3 * bl3) as u64;
		hi += (ah3 * bh3) as u64;
		lo += (al2 * bl4) as u64;
		mid += (al2 * bh4) as u64;
		mid += (ah2 * bl4) as u64;
		hi += (ah2 * bh4) as u64;
		lo += (al1 * bl5) as u64;
		mid += (al1 * bh5) as u64;
		mid += (ah1 * bl5) as u64;
		hi += (ah1 * bh5) as u64;
		lo += (al0 * bl6) as u64;
		mid += (al0 * bh6) as u64;
		mid += (ah0 * bl6) as u64;
		hi += (ah0 * bh6) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[6] = word as u32;
		// k = 7
		lo = (al7 * bl0) as u64;
		mid = (al7 * bh0) as u64;
		mid += (ah7 * bl0) as u64;
		hi = (ah7 * bh0) as u64;
		lo += (al6 * bl1) as u64;
		mid += (al6 * bh1) as u64;
		mid += (ah6 * bl1) as u64;
		hi += (ah6 * bh1) as u64;
		lo += (al5 * bl2) as u64;
		mid += (al5 * bh2) as u64;
		mid += (ah5 * bl2) as u64;
		hi += (ah5 * bh2) as u64;
		lo += (al4 * bl3) as u64;
		mid += (al4 * bh3) as u64;
		mid += (ah4 * bl3) as u64;
		hi += (ah4 * bh3) as u64;
		lo += (al3 * bl4) as u64;
		mid += (al3 * bh4) as u64;
		mid += (ah3 * bl4) as u64;
		hi += (ah3 * bh4) as u64;
		lo += (al2 * bl5) as u64;
		mid += (al2 * bh5) as u64;
		mid += (ah2 * bl5) as u64;
		hi += (ah2 * bh5) as u64;
		lo += (al1 * bl6) as u64;
		mid += (al1 * bh6) as u64;
		mid += (ah1 * bl6) as u64;
		hi += (ah1 * bh6) as u64;
		lo += (al0 * bl7) as u64;
		mid += (al0 * bh7) as u64;
		mid += (ah0 * bl7) as u64;
		hi += (ah0 * bh7) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[7] = word as u32;
		// k = 8
		lo = (al7 * bl1) as u64;
		mid = (al7 * bh1) as u64;
		mid += (ah7 * bl1) as u64;
		hi = (ah7 * bh1) as u64;
		lo += (al6 * bl2) as u64;
		mid += (al6 * bh2) as u64;
		mid += (ah6 * bl2) as u64;
		hi += (ah6 * bh2) as u64;
		lo += (al5 * bl3) as u64;
		mid += (al5 * bh3) as u64;
		mid += (ah5 * bl3) as u64;
		hi += (ah5 * bh3) as u64;
		lo += (al4 * bl4) as u64;
		mid += (al4 * bh4) as u64;
		mid += (ah4 * bl4) as u64;
		hi += (ah4 * bh4) as u64;
		lo += (al3 * bl5) as u64;
		mid += (al3 * bh5) as u64;
		mid += (ah3 * bl5) as u64;
		hi += (ah3 * bh5) as u64;
		lo += (al2 * bl6) as u64;
		mid += (al2 * bh6) as u64;
		mid += (ah2 * bl6) as u64;
		hi += (ah2 * bh6) as u64;
		lo += (al1 * bl7) as u64;
		mid += (al1 * bh7) as u64;
		mid += (ah1 * bl7) as u64;
		hi += (ah1 * bh7) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[8] = word as u32;
		// k = 9
		lo = (al7 * bl2) as u64;
		mid = (al7 * bh2) as u64;
		mid += (ah7 * bl2) as u64;
		hi = (ah7 * bh2) as u64;
		lo += (al6 * bl3) as u64;
		mid += (al6 * bh3) as u64;
		mid += (ah6 * bl3) as u64;
		hi += (ah6 * bh3) as u64;
		lo += (al5 * bl4) as u64;
		mid += (al5 * bh4) as u64;
		mid += (ah5 * bl4) as u64;
		hi += (ah5 * bh4) as u64;
		lo += (al4 * bl5) as u64;
		mid += (al4 * bh5) as u64;
		mid += (ah4 * bl5) as u64;
		hi += (ah4 * bh5) as u64;
		lo += (al3 * bl6) as u64;
		mid += (al3 * bh6) as u64;
		mid += (ah3 * bl6) as u64;
		hi += (ah3 * bh6) as u64;
		lo += (al2 * bl7) as u64;
		mid += (al2 * bh7) as u64;
		mid += (ah2 * bl7) as u64;
		hi += (ah2 * bh7) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[9] = word as u32;
		// k = 10
		lo = (al7 * bl3) as u64;
		mid = (al7 * bh3) as u64;
		mid += (ah7 * bl3) as u64;
		hi = (ah7 * bh3) as u64;
		lo += (al6 * bl4) as u64;
		mid += (al6 * bh4) as u64;
		mid += (ah6 * bl4) as u64;
		hi += (ah6 * bh4) as u64;
		lo += (al5 * bl5) as u64;
		mid += (al5 * bh5) as u64;
		mid += (ah5 * bl5) as u64;
		hi += (ah5 * bh5) as u64;
		lo += (al4 * bl6) as u64;
		mid += (al4 * bh6) as u64;
		mid += (ah4 * bl6) as u64;
		hi += (ah4 * bh6) as u64;
		lo += (al3 * bl7) as u64;
		mid += (al3 * bh7) as u64;
		mid += (ah3 * bl7) as u64;
		hi += (ah3 * bh7) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[10] = word as u32;
		// k = 11
		lo = (al7 * bl4) as u64;
		mid = (al7 * bh4) as u64;
		mid += (ah7 * bl4) as u64;
		hi = (ah7 * bh4) as u64;
		lo += (al6 * bl5) as u64;
		mid += (al6 * bh5) as u64;
		mid += (ah6 * bl5) as u64;
		hi += (ah6 * bh5) as u64;
		lo += (al5 * bl6) as u64;
		mid += (al5 * bh6) as u64;
		mid += (ah5 * bl6) as u64;
		hi += (ah5 * bh6) as u64;
		lo += (al4 * bl7) as u64;
		mid += (al4 * bh7) as u64;
		mid += (ah4 * bl7) as u64;
		hi += (ah4 * bh7) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[11] = word as u32;
		// k = 12
		lo = (al7 * bl5) as u64;
		mid = (al7 * bh5) as u64;
		mid += (ah7 * bl5) as u64;
		hi = (ah7 * bh5) as u64;
		lo += (al6 * bl6) as u64;
		mid += (al6 * bh6) as u64;
		mid += (ah6 * bl6) as u64;
		hi += (ah6 * bh6) as u64;
		lo += (al5 * bl7) as u64;
		mid += (al5 * bh7) as u64;
		mid += (ah5 * bl7) as u64;
		hi += (ah5 * bh7) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[12] = word as u32;
		// k = 13
		lo = (al7 * bl6) as u64;
		mid = (al7 * bh6) as u64;
		mid += (ah7 * bl6) as u64;
		hi = (ah7 * bh6) as u64;
		lo += (al6 * bl7) as u64;
		mid += (al6 * bh7) as u64;
		mid += (ah6 * bl7) as u64;
		hi += (ah6 * bh7) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[13] = word as u32;
		// k = 14
		lo = (al7 * bl7) as u64;
		mid = (al7 * bh7) as u64;
		mid += (ah7 * bl7) as u64;
		hi = (ah7 * bh7) as u64;
		word = c + lo + ((mid & 0xffff) << 16);
		c = hi + (mid >> 16) + (word >> 32);
		self.words[14] = word as u32;
		self.len = 15;
		if c != 0 {
			self.words[15] = c as u32;
			self.len = 16;
		}
	}

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
		self >= N
	}

	#[inline]
	pub fn is_even(&self) -> bool {
		self.words[0] & 1 == 0
	}

	#[inline]
	pub fn is_odd(&self) -> bool {
		self.words[0] & 1 == 1
	}

	pub fn split(&mut self) -> BigNum {
		if self.len < 9 {
			return ZERO;
		}

		let mut high = BigNum {
			negative: false,
			words: [0; 16],
			len: self.len - 8
		};

		high.words[0..8].copy_from_slice(&self.words[8..16]);
		self.len = 8;

		high
	}

	pub fn write_bytes_to(&self, buf: &mut [u8]) {
		write_u32(self.words[0], &mut buf[28..32]);
		write_u32(self.words[1], &mut buf[24..28]);
		write_u32(self.words[2], &mut buf[20..24]);
		write_u32(self.words[3], &mut buf[16..20]);
		write_u32(self.words[4], &mut buf[12..16]);
		write_u32(self.words[5], &mut buf[8..12]);
		write_u32(self.words[6], &mut buf[4..8]);
		write_u32(self.words[7], &mut buf[0..4]);
	}

	pub fn double(&mut self) {
		let mut i = 0;
		let mut carry = 0u64;

		for word in self.words_mut() {
			let w = (*word as u64) * 2 + carry;
			*word = w as u32;
			carry = w >> 32;
		}

		if carry != 0 {
			self.words[self.len] = carry as u32;
			self.len += 1;
		}
	}

	pub fn get_naf1(&self) -> NAFRepr {
		let mut naf = NAFRepr::new();

		let mut k = *self;

		while k != 0 {
			let zeros = k.words[0].trailing_zeros();

			if zeros != 0 {
				naf.push_zeros(zeros as usize);
				k >>= zeros;
				continue;
			}

			let m = (k.words[0] as i32) & 3;

			if m == 3 {
				naf.push(-1);
				naf.push_zeros(1);
				k += 1;
				k >>= 2;
			} else {
				naf.push(m as i8);
				k.words[0] -= m as u32;

				if k != 0 {
					k >>= 1;
				}
			}
		}

		naf
	}

	pub fn get_naf(&self, w: u8) -> NAF {
		let mut naf = NAF::new();
		let ws = 1i32 << (w + 1);
		let wsm1 = ws - 1;
		let ws2 = ws / 2;

		let mut k = *self;

		while k != 0 {
			let zeros = k.words[0].trailing_zeros();

			if zeros != 0 {
				naf.push_zeros(zeros as usize);
				k >>= zeros;
				continue;
			}

			let m = (k.words[0] as i32) & wsm1;

			if m > ws2 {
				naf.push((ws2 - m) as i8);

				k += (m - ws2) as u32;
				k >>= 1;
			} else {
				naf.push(m as i8);
				k.words[0] -= m as u32;

				if k != 0 {
					naf.push_zeros(w as usize - 1);
					k >>= w as u32;
				}
			}
		}

		naf
	}

	pub fn red_neg(&self) -> BigNum {
		if self == 0 {
			ZERO
		} else {
			*P - self
		}
	}

	pub fn red_add(&self, num: &BigNum) -> BigNum {
		let mut res = *self + num;

		if &res >= P {
			res -= P
		}

		res
	}

	pub fn red_double(&mut self) {
		self.double();

		if &*self >= P {
			self.sub_assign(P);
		}
	}

	pub fn red_add_mut(&mut self, num: &BigNum) {
		self.add_assign(num);

		if &*self >= P {
			self.sub_assign(P);
		}
	}

	pub fn red_sub_twice(&self, num: &BigNum) -> BigNum {
		let mut res = *self - num;
		res -= num;

		while res.negative {
			res += P;
		}

		res
	}

	pub fn red_sub(&self, num: &BigNum) -> BigNum {
		let mut res = *self - num;

		if res.negative {
			res += P
		}

		res
	}

	pub fn red_mul(&self, num: &BigNum) -> BigNum {
		let mut res = *self * num;
		res.red_reduce();
		res
	}

	#[inline]
	pub fn red_mul_mut(&mut self, num: &BigNum) {
		self.mul_assign(num);
		self.red_reduce();
	}

	pub fn red_invm(mut self) -> BigNum {
		let mut b = *P;

		let mut x1 = ONE;
		let mut x2 = ZERO;

		while self > 1 && b > 1 {
			let a_zeros = self.words[0].trailing_zeros();

			if a_zeros != 0 {
				self >>= a_zeros;
				for _ in 0..a_zeros {
					if x1.is_odd() {
						x1 += P;
					}
					x1 >>= 1;
				}
			}

			let b_zeros = b.words[0].trailing_zeros();
			if b_zeros != 0 {
				b >>= b_zeros;
				for _ in 0..b_zeros {
					if x2.is_odd() {
						x2 += P;
					}
					x2 >>= 1;
				}
			}

			if self >= b {
				self -= &b;
				x1 -= &x2;
			} else {
				b -= &self;
				x2 -= &x1;
			}
		}

		if self == 1 {
			self = x1;
		} else {
			self = x2;
		}

		if self.negative {
			self += P;
		}

		if self.negative {
			self.negative = false;
			self.red_reduce();
			self.red_neg()
		} else {
			self.red_reduce();
			self
		}
	}

	pub fn red_sqr(&self) -> BigNum {
		let mut res = *self * self;
		res.red_reduce();
		res
	}

	pub fn mul_k(&mut self) {
		self.words[self.len] = 0;
		self.words[self.len + 1] = 0;
		self.len += 2;

		let mut low = 0;

		// k is 0x1000003d1 (does not fit into u32)
		// by performing the extra addition of w
		// we avoid potential math overflows

		for word in self.words_mut() {
			let w = *word as u64;
			low += w * 0x3d1;
			*word = low as u32;
			low = w + (low >> 32); // w * 1 + (low >> 32)
		}

		self.strip();
	}

	pub fn red_reduce(&mut self) {
		let mut high = self.split();

		high.mul_k();
		self.add_assign(&high);

		if self.len > 8 {
			let mut high = self.split();

			high.mul_k();
			self.add_assign(&high);
		}

		match (&*self).cmp(P) {
			Ordering::Equal => {},
			Ordering::Greater => self.sub_assign(P),
			Ordering::Less => self.strip()
		}
	}
}

pub const ZERO: BigNum = BigNum {
	negative: false,
	len: 1,
	words: [0; 16]
};

pub const ONE: BigNum = BigNum {
	negative: false,
	len: 1,
	words: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
};

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
pub static N: &'static BigNum = &BigNum {
	negative: false,
	len: 8,
	words: [
		0xd0364141,
		0xbfd25e8c,
		0xaf48a03b,
		0xbaaedce6,
		0xfffffffe,
		0xffffffff,
		0xffffffff,
		0xffffffff,
		0, 0, 0, 0, 0, 0, 0, 0
	]
};

// 7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0
pub static NH: &'static BigNum = &BigNum {
	negative: false,
	len: 8,
	words: [
		0x681b20a0,
		0xdfe92f46,
		0x57a4501d,
		0x5d576e73,
		0xffffffff,
		0xffffffff,
		0xffffffff,
		0x7fffffff,
		0, 0, 0, 0, 0, 0, 0, 0
	]
};

// 000000000000000000000000000000014551231950B75FC4402DA1732FC9BEBF
pub static NC: &'static BigNum = &BigNum {
	negative: false,
	len: 5,
	words: [
		0x2fc9bebf,
		0x402da173,
		0x50b75fc4,
		0x45512319,
		0x00000001,
		0x00000000,
		0x00000000,
		0x00000000,
		0, 0, 0, 0, 0, 0, 0, 0
	]
};

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
pub static P: &'static BigNum = &BigNum {
	negative: false,
	len: 8,
	words: [
		0xfffffc2f,
		0xfffffffe,
		0xffffffff,
		0xffffffff,
		0xffffffff,
		0xffffffff,
		0xffffffff,
		0xffffffff,
		0, 0, 0, 0, 0, 0, 0, 0
	]
};

// P - N = 000000000000000000000000000000014551231950b75fc4402da1722fc9baee,
pub static PSN: &'static BigNum = &BigNum {
	negative: false,
	len: 5,
	words: [
		0x2fc9baee,
		0x402da172,
		0x50b75fc4,
		0x45512319,
		0x00000001,
		0x00000000,
		0x00000000,
		0x00000000,
		0, 0, 0, 0, 0, 0, 0, 0
	]
};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn big_num_add_big_num() {
		let n = *NC + NC;

		let mut nm = *NC;
		nm += NC;

		let expected_bytes: &[u8] = &[
			0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
			0x00,0x00,0x02,0x8a,0xa2,0x46,0x32,0xa1,0x6e,0xbf,0x88,0x80,0x5b,
			0x42,0xe6,0x5f,0x93,0x7d,0x7e
		];

		assert_eq!(n, BigNum::from(expected_bytes));
		assert_eq!(nm, BigNum::from(expected_bytes));
	}

	#[test]
	fn big_num_sub_big_num() {
		let n = *P - NC;

		let expected_bytes: &[u8] = &[
			0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
			0xff,0xff,0xfe,0xba,0xae,0xdc,0xe6,0xaf,0x48,0xa0,0x3b,0xbf,0xd2,
			0x5e,0x8b,0xd0,0x36,0x3d,0x70
		];

		assert_eq!(n, BigNum::from(expected_bytes));
	}

	#[test]
	fn big_num_sub_big_num_negative() {
		let n = ONE - NC;

		let expected_bytes: &[u8] = &[
			0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
			0x00,0x00,0x01,0x45,0x51,0x23,0x19,0x50,0xb7,0x5f,0xc4,0x40,0x2d,
			0xa1,0x73,0x2f,0xc9,0xbe,0xbe,
		];
		let mut expected = BigNum::from(expected_bytes);
		expected.negative = true;

		assert_eq!(n, expected);
	}

	#[test]
	fn big_num_shr() {
		let n = *NC >> 7;

		let mut nm = *NC;
		nm >>= 7;

		let expected_bytes: &[u8] = &[
			0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
			0x00,0x00,0x00,0x02,0x8a,0xa2,0x46,0x32,0xa1,0x6e,0xbf,0x88,0x80,
			0x5b,0x42,0xe6,0x5f,0x93,0x7d
		];

		assert_eq!(n, BigNum::from(expected_bytes));
		assert_eq!(nm, BigNum::from(expected_bytes));
	}

	#[test]
	fn produces_valid_n() {
		let bytes: &[u8] = &[
			0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
			0xFF,0xFF,0xFE,0xBA,0xAE,0xDC,0xE6,0xAF,0x48,0xA0,0x3B,0xBF,0xD2,
			0x5E,0x8C,0xD0,0x36,0x41,0x41
		];

		let mut roundtrip = [0u8; 32];
		let bn = BigNum::from(bytes);
		bn.write_bytes_to(&mut roundtrip);

		assert_eq!(&bn, N);
		assert_eq!(bytes, roundtrip);
	}

	#[test]
	fn produces_valid_nh() {
		let nh = *N >> 1;

		assert_eq!(&nh, NH);
	}

	#[test]
	fn produces_valid_nc() {
		let bytes: &[u8] = &[
			0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
			0x00,0x00,0x01,0x45,0x51,0x23,0x19,0x50,0xB7,0x5F,0xC4,0x40,0x2D,
			0xA1,0x73,0x2F,0xC9,0xBE,0xBF
		];

		let mut roundtrip = [0u8; 32];
		let bn = BigNum::from(bytes);
		bn.write_bytes_to(&mut roundtrip);

		assert_eq!(&bn, NC);
		assert_eq!(bytes, roundtrip);
	}

	#[test]
	fn produces_valid_p() {
		let bytes: &[u8] = &[
			0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
			0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
			0xFF,0xFE,0xFF,0xFF,0xFC,0x2F
		];

		let mut roundtrip = [0u8; 32];
		let bn = BigNum::from(bytes);
		bn.write_bytes_to(&mut roundtrip);

		assert_eq!(&bn, P);
		assert_eq!(bytes, roundtrip);
	}

	#[test]
	fn produces_valid_psn() {
		let psn = *P - &N;

		assert_eq!(&psn, PSN);
	}

	#[test]
	fn big_num_partial_eq_u32() {
		assert!(ZERO == 0);
		assert!(ONE == 1);
		assert!(N != 0);
		assert!(N != 1);
	}

	#[test]
	fn big_num_partial_ord_u32() {
		let five = BigNum::from(5);

		assert!(five > 4);
		assert!(five >= 4);
		assert!(five >= 5);
		assert!(five == 5);
		assert!(five <= 5);
		assert!(five <= 6);
		assert!(five < 6);
	}

	#[test]
	fn is_overflow() {
		let np1 = *N + &BigNum::from(1);
		let ns1 = *N - &BigNum::from(1);

		assert_eq!(np1.is_overflow(), true);
		assert_eq!(N.is_overflow(), true);
		assert_eq!(ns1.is_overflow(), false);
	}

	#[test]
	fn multiply() {
		let mut low = (*N * NC);
		let high = low.split();

		let expected_bytes: &[u8] = &[
			0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
			0x00,0x00,0x01,0x45,0x51,0x23,0x19,0x50,0xB7,0x5F,0xC4,0x40,0x2D,
			0xA1,0x73,0x2F,0xC9,0xBE,0xBD,0x62,0x98,0xE3,0x2A,0x7E,0x39,0x64,
			0x3A,0x19,0x68,0x0A,0x1B,0xA4,0x32,0xF8,0x3A,0xD1,0x3C,0x8C,0x57,
			0x42,0x3A,0x67,0x4B,0xB6,0xC0,0xAF,0x5E,0xC7,0xF1,0xED,0x7F
		];

		let mut buf = [0u8; 64];

		low.write_bytes_to(&mut buf[32..]);
		high.write_bytes_to(&mut buf[..32]);

		assert_eq!(&buf[..], expected_bytes);
	}

	#[test]
	fn mul_k() {
		let mut n = *NC;
		n.mul_k();

		let expected_bytes: &[u8] = &[
			0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01,0x45,
			0x51,0x27,0xf2,0xdb,0x5e,0x53,0x61,0x4c,0x02,0x1d,0x6c,0x1d,0xee,
			0xe7,0x58,0x60,0xf0,0xf6,0xef
		];

		assert_eq!(n, BigNum::from(expected_bytes));
	}

	#[test]
	fn red_reduce() {
		let mut reduced = (*N * NC);
		reduced.red_reduce();

		let expected_bytes: &[u8] = &[
			0x62,0x98,0xe3,0x2a,0x7e,0x39,0x64,0x3a,0x19,0x68,0x0a,0x1c,0xe9,
			0x84,0x20,0x2d,0xac,0x9a,0xdf,0xb8,0x8e,0x3c,0x84,0xb7,0xd4,0xaf,
			0x96,0xb5,0x28,0xe2,0xdc,0xcc
		];

		assert_eq!(reduced, BigNum::from(expected_bytes));
	}

	#[test]
	fn red_neg() {
		let n = NC.red_neg();

		let expected_bytes: &[u8] = &[
			0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
			0xff,0xff,0xfe,0xba,0xae,0xdc,0xe6,0xaf,0x48,0xa0,0x3b,0xbf,0xd2,
			0x5e,0x8b,0xd0,0x36,0x3d,0x70
		];

		assert_eq!(n, BigNum::from(expected_bytes));
	}

	#[test]
	fn red_invm() {
		let n = NC.red_invm();

		let expected_bytes: &[u8] = &[
			0x47,0x83,0x3b,0x08,0x4c,0xa6,0x29,0x77,0xde,0xde,0x0f,0xd2,0xd9,
			0x03,0xba,0x08,0x2d,0x2f,0x64,0x1f,0x84,0x5f,0x50,0x59,0xf7,0x16,
			0xdf,0x89,0x80,0x6e,0x26,0xd1
		];

		assert_eq!(n, BigNum::from(expected_bytes));
	}

	#[test]
	fn n_sub_one() {
		let bn = *N - &BigNum::from(1);

		let expected_bytes: &[u8] = &[
			0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
			0xff,0xff,0xfe,0xba,0xae,0xdc,0xe6,0xaf,0x48,0xa0,0x3b,0xbf,0xd2,
			0x5e,0x8c,0xd0,0x36,0x41,0x40
		];

		assert_eq!(bn, BigNum::from(expected_bytes));
	}

	#[test]
	fn n_naf() {
		let expected_naf_1: &[i8] = &[
			1,0,0,0,0,0,1,0,1,0,0,0,0,0,1,0,0,-1,0,-1,0,0,1,0,0,0,0,0,1,0,-1,0,
			1,0,-1,0,1,0,0,1,0,-1,0,0,0,-1,0,1,0,1,0,0,1,0,-1,0,0,0,0,0,0,0,-1,
			0,0,0,-1,0,0,0,1,0,0,0,0,0,0,1,0,1,0,0,0,1,0,0,1,0,-1,0,0,0,-1,0,
			-1,0,-1,0,0,1,0,-1,0,0,1,0,-1,0,0,-1,0,0,-1,0,0,0,-1,0,-1,0,-1,0,
			-1,0,0,0,-1,0,-1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,1
		];
		let expected_naf_7: &[i8] = &[
			65,0,0,0,0,0,0,0,65,0,0,0,0,0,0,0,0,27,0,0,0,0,0,0,0,0,0,0,-77,-13,
			77,0,0,0,0,0,0,0,0,-61,125,0,0,0,0,0,0,0,0,-105,41,0,0,0,0,0,0,0,0,
			0,0,0,-111,-47,111,0,0,0,0,0,0,0,0,0,0,0,0,69,0,0,0,0,0,0,0,0,-61,
			125,0,0,0,0,0,0,0,-77,13,0,0,0,0,0,0,0,-93,-29,-93,29,0,0,0,0,0,0,
			0,0,-43,-107,43,0,0,0,0,0,0,0,-123,59,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1
		];

		assert_eq!(expected_naf_1, N.get_naf(1).as_slice());
		assert_eq!(expected_naf_7, N.get_naf(7).as_slice());
	}

	#[test]
	fn p_naf() {
		let expected_naf_1: &[i8] = &[
			-1,0,0,0,-1,0,1,0,0,0,-1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			-1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1
		];
		let expected_naf_7: &[i8] = &[
			47,0,0,0,0,0,0,0,0,0,-127,63,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,-127,63,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
			0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1
		];

		assert_eq!(expected_naf_1, P.get_naf(1).as_slice());
		assert_eq!(expected_naf_7, P.get_naf(7).as_slice());
	}
}
