# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import json
import orjson
import rapidjson
import ujson

from .util import read_fixture_obj, read_fixture_str


def test_dumps_canada_orjson(benchmark):
    benchmark.group = 'canada.json serialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_obj("canada.json.xz")
    benchmark(orjson.dumps, data)

def test_dumps_canada_ujson(benchmark):
    benchmark.group = 'canada.json serialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_obj("canada.json.xz")
    benchmark(ujson.dumps, data)

def test_dumps_canada_json(benchmark):
    benchmark.group = 'canada.json serialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_obj("canada.json.xz")
    benchmark(json.dumps, data)

def test_dumps_canada_rapidjson(benchmark):
    benchmark.group = 'canada.json serialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_obj("canada.json.xz")
    benchmark(rapidjson.dumps, data)

def test_dumps_citm_catalog_orjson(benchmark):
    benchmark.group = 'citm_catalog.json serialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_obj("citm_catalog.json.xz")
    benchmark(orjson.dumps, data)

def test_dumps_citm_catalog_ujson(benchmark):
    benchmark.group = 'citm_catalog.json serialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_obj("citm_catalog.json.xz")
    benchmark(ujson.dumps, data)

def test_dumps_citm_catalog_json(benchmark):
    benchmark.group = 'citm_catalog.json serialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_obj("citm_catalog.json.xz")
    benchmark(json.dumps, data)

def test_dumps_citm_catalog_rapidjson(benchmark):
    benchmark.group = 'citm_catalog.json serialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_obj("citm_catalog.json.xz")
    benchmark(rapidjson.dumps, data)

def test_dumps_github_orjson(benchmark):
    benchmark.group = 'github.json serialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_obj("github.json.xz")
    benchmark(orjson.dumps, data)

def test_dumps_github_ujson(benchmark):
    benchmark.group = 'github.json serialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_obj("github.json.xz")
    benchmark(ujson.dumps, data)

def test_dumps_github_json(benchmark):
    benchmark.group = 'github.json serialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_obj("github.json.xz")
    benchmark(json.dumps, data)

def test_dumps_github_rapidjson(benchmark):
    benchmark.group = 'github.json serialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_obj("github.json.xz")
    benchmark(rapidjson.dumps, data)

def test_dumps_twitter_orjson(benchmark):
    benchmark.group = 'twitter.json serialization'
    benchmark.extra_info['lib'] = 'orjson'
    data = read_fixture_obj("twitter.json.xz")
    benchmark(orjson.dumps, data)

def test_dumps_twitter_ujson(benchmark):
    benchmark.group = 'twitter.json serialization'
    benchmark.extra_info['lib'] = 'ujson'
    data = read_fixture_obj("twitter.json.xz")
    benchmark(ujson.dumps, data)

def test_dumps_twitter_json(benchmark):
    benchmark.group = 'twitter.json serialization'
    benchmark.extra_info['lib'] = 'json'
    data = read_fixture_obj("twitter.json.xz")
    benchmark(json.dumps, data)

def test_dumps_twitter_rapidjson(benchmark):
    benchmark.group = 'twitter.json serialization'
    benchmark.extra_info['lib'] = 'rapidjson'
    data = read_fixture_obj("twitter.json.xz")
    benchmark(rapidjson.dumps, data)
