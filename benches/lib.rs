#![feature(test)]

extern crate tiny_secp256k1;
extern crate test;

use tiny_secp256k1::{ECPointG, create_public_key};

use test::Bencher;

#[bench]
fn precalculate_ecpoint_g(b: &mut Bencher) {
    b.iter(|| {
        ECPointG::new()
    });
}

#[bench]
fn secret_to_public(b: &mut Bencher) {
    let g = ECPointG::new();
    let secret: &[u8] = &[
        0x32, 0x79, 0xe8, 0x0c, 0xb3, 0x93, 0x5c, 0x68, 0xdc, 0xf3, 0x71, 0xb9,
        0xee, 0x21, 0x78, 0x73, 0x84, 0xba, 0xee, 0x63, 0xd6, 0x49, 0x0b, 0x17,
        0x39, 0x27, 0x10, 0xc8, 0x76, 0xb1, 0xa8, 0x6b
    ];

    b.iter(|| {
        create_public_key(&g, secret)
    });
}
