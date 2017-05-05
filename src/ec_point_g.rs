use big_num::BigNum;
use ecj_point::ECJPoint;
use ec_point::{self, ECPoint};
use naf::NAFPoints;

pub struct ECPointG {
    ecpoint: ECPoint,
    naf: NAFPoints,
    points: [ECPoint; 66],
    negpoints: [ECPoint; 66]
}

impl ECPointG {
    /// This can be very expensive and should  be performed only once.
    /// Ideally this would be memoized (using lazy_static or otherwise),
    /// however no_std makes that difficult, and using static would
    /// bloat up the WASM binary size.
    pub fn new() -> Self {
        let x: &[u8] = &[
            0x79,0xBE,0x66,0x7E,0xF9,0xDC,0xBB,0xAC,0x55,0xA0,0x62,0x95,0xCE,
            0x87,0x0B,0x07,0x02,0x9B,0xFC,0xDB,0x2D,0xCE,0x28,0xD9,0x59,0xF2,
            0x81,0x5B,0x16,0xF8,0x17,0x98
        ];
        let y: &[u8] = &[
            0x48,0x3A,0xDA,0x77,0x26,0xA3,0xC4,0x65,0x5D,0xA4,0xFB,0xFC,0x0E,
            0x11,0x08,0xA8,0xFD,0x17,0xB4,0x48,0xA6,0x85,0x54,0x19,0x9C,0x47,
            0xD0,0x8F,0xFB,0x10,0xD4,0xB8
        ];

        let mut acc = ECPoint::new(x.into(), y.into());

        // dstep = 4
        // points.len = 1 + (257 / dstep) = 66

        let mut res = ECPointG {
            ecpoint: acc,
            naf: NAFPoints::new(7, acc),
            points: [acc; 66],
            negpoints: [acc.neg(); 66]
        };

        for (point, negpoint) in res.points[1..]
            .iter_mut()
            .zip(res.negpoints[1..].iter_mut())
        {
            // dstep doubles
            acc.double();
            acc.double();
            acc.double();
            acc.double();
            *point = acc;
            *negpoint = acc.neg();
        }

        res
    }

    pub fn mul(&self, num: &mut BigNum) -> ECPoint {
        let naf = num.get_naf1();
        let repr = naf.as_slice();

        let mut a = ECJPoint::default();
        let mut b = ECJPoint::default();

        // I = ((1 << (step + 1)) - (step % 2 === 0 ? 2 : 1)) / 3 == 10
        let mut i = 10;

        while i > 0 {
            for (ri, rval) in repr.iter().enumerate() {
                if rval == &i {
                    b.mixed_add(&self.points[ri]);
                } else if rval == &-i {
                    b.mixed_add(&self.negpoints[ri]);
                }
            }

            a += &b;
            i -= 1;
        }

        a.into()
    }
}

#[cfg(test)]
mod tests {
}
