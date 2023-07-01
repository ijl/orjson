export CC="clang"
export CFLAGS="-O2 -fno-plt -flto=thin"
export LDFLAGS="-O2 -flto=thin -fuse-ld=lld -Wl,--as-needed"
export RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=lld"