#!/bin/bash
cargo test -j 2 --manifest-path zome/Cargo.toml --lib --features="mock" -- --nocapture