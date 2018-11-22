# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import json
import orjson
import rapidjson
import ujson

from .util import read_fixture_obj, read_fixture_str


def test_loads_canada_orjson(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_str("canada.json.xz")
    benchmark(orjson.loads, data)

def test_loads_canada_ujson(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_str("canada.json.xz")
    benchmark(ujson.loads, data)

def test_loads_canada_json(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_str("canada.json.xz")
    benchmark(json.loads, data)

def test_loads_canada_rapidjson(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_str("canada.json.xz")
    benchmark(json.loads, data)

def test_loads_citm_catalog_orjson(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(orjson.loads, data)

def test_loads_citm_catalog_ujson(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(ujson.loads, data)

def test_loads_citm_catalog_json(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(json.loads, data)

def test_loads_citm_catalog_rapidjson(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(json.loads, data)

def test_loads_twitter_orjson(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_str("twitter.json.xz")
    benchmark(orjson.loads, data)

def test_loads_twitter_ujson(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_str("twitter.json.xz")
    benchmark(ujson.loads, data)

def test_loads_twitter_json(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_str("twitter.json.xz")
    benchmark(json.loads, data)

def test_loads_twitter_rapidjson(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_str("twitter.json.xz")
    benchmark(json.loads, data)
