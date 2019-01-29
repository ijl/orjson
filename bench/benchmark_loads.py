# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

from json import loads as json_loads
from orjson import loads as orjson_loads
from rapidjson import loads as rapidjson_loads
from simplejson import loads as simplejson_loads
from ujson import loads as ujson_loads

from .util import read_fixture_obj, read_fixture_str


def test_loads_canada_orjson(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_str("canada.json.xz")
    benchmark(orjson_loads, data)

def test_loads_canada_ujson(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_str("canada.json.xz")
    benchmark(ujson_loads, data)

def test_loads_canada_json(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_str("canada.json.xz")
    benchmark(json_loads, data)

def test_loads_canada_rapidjson(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_str("canada.json.xz")
    benchmark(rapidjson_loads, data)

def test_loads_canada_simplejson(benchmark):
    benchmark.group = 'canada.json deserialization'
    benchmark.extra_info['lib'] = 'simplejson'
    data = read_fixture_str("canada.json.xz")
    benchmark(simplejson_loads, data)

def test_loads_citm_catalog_orjson(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(orjson_loads, data)

def test_loads_citm_catalog_ujson(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(ujson_loads, data)

def test_loads_citm_catalog_json(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(json_loads, data)

def test_loads_citm_catalog_rapidjson(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(rapidjson_loads, data)

def test_loads_citm_catalog_simplejson(benchmark):
    benchmark.group = 'citm_catalog.json deserialization'
    benchmark.extra_info['lib'] = 'simplejson'
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark(simplejson_loads, data)

def test_loads_github_orjson(benchmark):
    benchmark.group = 'github.json deserialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_str("github.json.xz")
    benchmark(orjson_loads, data)

def test_loads_github_ujson(benchmark):
    benchmark.group = 'github.json deserialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_str("github.json.xz")
    benchmark(ujson_loads, data)

def test_loads_github_json(benchmark):
    benchmark.group = 'github.json deserialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_str("github.json.xz")
    benchmark(json_loads, data)

def test_loads_github_rapidjson(benchmark):
    benchmark.group = 'github.json deserialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_str("github.json.xz")
    benchmark(rapidjson_loads, data)

def test_loads_github_simplejson(benchmark):
    benchmark.group = 'github.json deserialization'
    benchmark.extra_info['lib'] = 'simplejson'
    data = read_fixture_str("github.json.xz")
    benchmark(simplejson_loads, data)

def test_loads_twitter_orjson(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_str("twitter.json.xz")
    benchmark(orjson_loads, data)

def test_loads_twitter_ujson(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_str("twitter.json.xz")
    benchmark(ujson_loads, data)

def test_loads_twitter_json(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_str("twitter.json.xz")
    benchmark(json_loads, data)

def test_loads_twitter_rapidjson(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_str("twitter.json.xz")
    benchmark(rapidjson_loads, data)

def test_loads_twitter_simplejson(benchmark):
    benchmark.group = 'twitter.json deserialization'
    benchmark.extra_info['lib'] = 'simplejson'
    data = read_fixture_str("twitter.json.xz")
    benchmark(simplejson_loads, data)
