# tiny-secp256k1

A pure-Rust `no_std` implementation of Secp256k1 with a primary goal of having a working solution that can be easily compiled to Web Assembly. The logic is mostly a port of [https://github.com/cryptocoinjs/secp256k1-node/tree/master/lib/js](node-secp256k1), adapting for extended precision of integers available in Rust (and WASM) where applicable.

The crate is currently only capable of verifying secret keys and generating public keys from secrets.
