# tiny-secp256k1

A pure-Rust `no_std` implementation of Secp256k1. A primary goal for this crate is having a working solution that can be easily compiled to WebAssembly.

The logic is mostly a port of [https://github.com/cryptocoinjs/secp256k1-node/tree/master/lib/js](node-secp256k1), adapting for extended precision of integers available in Rust (and WASM) where applicable.

The crate is currently only capable of verifying secret keys and generating public keys from secrets.
