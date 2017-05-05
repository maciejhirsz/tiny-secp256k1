use ec_point::{ECPoint, INF};
use core::mem;

pub struct NAFRepr {
    data: [i8; 128],
    cur: usize,
    bit: usize,
}

impl NAFRepr {
    pub fn new() -> Self {
        NAFRepr {
            data: [0; 128],
            cur: 0,
            bit: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cur + if self.bit != 0 { 1 } else { 0 }
    }

    #[inline]
    pub fn push(&mut self, val: i8) {
        self.data[self.cur] |= val << self.bit;
        self.bit += 1;

        if self.bit == 4 {
            self.bit = 0;
            self.cur += 1;
        }
    }

    #[inline]
    pub fn push_zeros(&mut self, mut count: usize) {
        count += self.bit;
        self.cur += count / 4;
        self.bit = count % 4;
    }

    #[inline]
    pub fn as_slice(&self) -> &[i8] {
        &self.data[..self.len()]
    }
}

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
    pub fn len(&self) -> usize {
        self.len
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

pub struct NAFPoints {
    wnd: usize,
    points: [ECPoint; 127],
    len: usize
}

impl NAFPoints {
    pub fn new(wnd: usize, mut init: ECPoint) -> NAFPoints {
        let mut res = NAFPoints {
            wnd: wnd,
            points: unsafe { mem::uninitialized() },
            len: (1 << wnd) - 1
        };

        res.points[0] = init;
        let mut double = init;
        double.double();

        for point in &mut res.points[1..res.len] {
            init += &double;
            *point = init;
        }

        res
    }

    #[inline]
    pub fn as_slice(&self) -> &[ECPoint] {
        &self.points[..self.len]
    }

    #[inline]
    pub fn wnd(&self) -> usize {
        self.wnd
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}
