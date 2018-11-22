#!/bin/sh -e

# usage: ./profile data/citm_catalog.json.xz loads

perf record -g --call-graph lbr ./bench/run_func "$@"
perf report --hierarchy
rm -f perf.data*
