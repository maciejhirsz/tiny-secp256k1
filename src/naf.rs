pub struct NAF {
	data: [i8; 512],
	len: usize
}

impl NAF {
	pub fn new() -> Self {
		NAF {
			data: [0; 512],
			len: 0
		}
	}

	#[inline]
	pub fn push(&mut self, val: i8) {
		self.data[self.len] = val;
		self.len += 1;
	}

	#[inline]
	pub fn push_zeros(&mut self, count: usize) {
		self.len += count;
	}

	#[inline]
	pub fn as_slice(&self) -> &[i8] {
		&self.data[..self.len]
	}
}
