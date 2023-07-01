#!/bin/bash

export CC="clang"
export CFLAGS="-O2 -fno-plt -flto=thin"
export LDFLAGS="-O2 -flto=thin -fuse-ld=lld -Wl,--as-needed"
export RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=lld"

maturin build
cargo fetch
mkdir .cargo
cp ci/sdist.toml .cargo/config.toml
cargo vendor include/cargo --versioned-dirs
maturin sdist --out=dist
CARGO_NET_OFFLINE="true" python3 -m pip install dist/orjson*.tar.gz
rm -r .cargo

