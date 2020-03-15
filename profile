#!/bin/sh -e

# usage: ./profile data/citm_catalog.json.xz loads

perf record -g --call-graph lbr --delay 250 ./bench/run_func "$@"
perf report --hierarchy --percent-limit 0.1
rm -f perf.data*
