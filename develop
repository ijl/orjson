#!/bin/sh -e

rm -f target/wheels/*

maturin build --no-sdist --manylinux off -i python3 --release "$@"

pip install --force $(find target/wheels -name "*cp3*")
