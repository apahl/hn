#!/bin/bash
set -e

mkdir -p bin
cargo build --release
mv target/release/hn bin/