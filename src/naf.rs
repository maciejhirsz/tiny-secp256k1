use ec_point::{ECPoint, INF};
use core::mem;

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

pub struct NAFPoints {
    wnd: usize,
    points: [ECPoint; 127],
    len: usize
}

impl NAFPoints {
    pub fn new(wnd: usize, init: ECPoint) -> NAFPoints {
        let mut res = NAFPoints {
            wnd: wnd,
            points: unsafe { mem::uninitialized() },
            len: (1 << wnd) - 1
        };

        res.points[0] = init;
        let dbl = init.dbl();

        for i in 1..res.len {
            res.points[i] = res.points[i - 1] + dbl;
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
