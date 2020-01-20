# SPDX-License-Identifier: (Apache-2.0 OR MIT)


from json import loads as json_loads, dumps as json_dumps
from orjson import loads as orjson_loads, dumps as orjson_dumps
from rapidjson import loads as rapidjson_loads, dumps as rapidjson_dumps
from simplejson import loads as simplejson_loads, dumps as simplejson_dumps
from ujson import loads as ujson_loads, dumps as ujson_dumps

from .util import read_fixture_str


def test_loads_canada_orjson(benchmark):
    benchmark.group = "canada.json deserialization"
    benchmark.extra_info["lib"] = "orjson"
    data = read_fixture_str("canada.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        orjson_dumps(orjson_loads(data))
    ) == json_loads(data)
    benchmark(orjson_loads, data)


def test_loads_canada_ujson(benchmark):
    benchmark.group = "canada.json deserialization"
    benchmark.extra_info["lib"] = "ujson"
    data = read_fixture_str("canada.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        ujson_dumps(ujson_loads(data))
    ) == json_loads(data)
    benchmark(ujson_loads, data)


def test_loads_canada_json(benchmark):
    benchmark.group = "canada.json deserialization"
    benchmark.extra_info["lib"] = "json"
    data = read_fixture_str("canada.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        json_dumps(json_loads(data))
    ) == json_loads(data)
    benchmark(json_loads, data)


def test_loads_canada_rapidjson(benchmark):
    benchmark.group = "canada.json deserialization"
    benchmark.extra_info["lib"] = "rapidjson"
    data = read_fixture_str("canada.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        rapidjson_dumps(rapidjson_loads(data))
    ) == json_loads(data)
    benchmark(rapidjson_loads, data)


def test_loads_canada_simplejson(benchmark):
    benchmark.group = "canada.json deserialization"
    benchmark.extra_info["lib"] = "simplejson"
    data = read_fixture_str("canada.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        simplejson_dumps(simplejson_loads(data))
    ) == json_loads(data)
    benchmark(simplejson_loads, data)


def test_loads_citm_catalog_orjson(benchmark):
    benchmark.group = "citm_catalog.json deserialization"
    benchmark.extra_info["lib"] = "orjson"
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        orjson_dumps(orjson_loads(data))
    ) == json_loads(data)
    benchmark(orjson_loads, data)


def test_loads_citm_catalog_ujson(benchmark):
    benchmark.group = "citm_catalog.json deserialization"
    benchmark.extra_info["lib"] = "ujson"
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        ujson_dumps(ujson_loads(data))
    ) == json_loads(data)
    benchmark(ujson_loads, data)


def test_loads_citm_catalog_json(benchmark):
    benchmark.group = "citm_catalog.json deserialization"
    benchmark.extra_info["lib"] = "json"
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        json_dumps(json_loads(data))
    ) == json_loads(data)
    benchmark(json_loads, data)


def test_loads_citm_catalog_rapidjson(benchmark):
    benchmark.group = "citm_catalog.json deserialization"
    benchmark.extra_info["lib"] = "rapidjson"
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        rapidjson_dumps(rapidjson_loads(data))
    ) == json_loads(data)
    benchmark(rapidjson_loads, data)


def test_loads_citm_catalog_simplejson(benchmark):
    benchmark.group = "citm_catalog.json deserialization"
    benchmark.extra_info["lib"] = "simplejson"
    data = read_fixture_str("citm_catalog.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        simplejson_dumps(simplejson_loads(data))
    ) == json_loads(data)
    benchmark(simplejson_loads, data)


def test_loads_github_orjson(benchmark):
    benchmark.group = "github.json deserialization"
    benchmark.extra_info["lib"] = "orjson"
    data = read_fixture_str("github.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        orjson_dumps(orjson_loads(data))
    ) == json_loads(data)
    benchmark(orjson_loads, data)


def test_loads_github_ujson(benchmark):
    benchmark.group = "github.json deserialization"
    benchmark.extra_info["lib"] = "ujson"
    data = read_fixture_str("github.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        ujson_dumps(ujson_loads(data))
    ) == json_loads(data)
    benchmark(ujson_loads, data)


def test_loads_github_json(benchmark):
    benchmark.group = "github.json deserialization"
    benchmark.extra_info["lib"] = "json"
    data = read_fixture_str("github.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        json_dumps(json_loads(data))
    ) == json_loads(data)
    benchmark(json_loads, data)


def test_loads_github_rapidjson(benchmark):
    benchmark.group = "github.json deserialization"
    benchmark.extra_info["lib"] = "rapidjson"
    data = read_fixture_str("github.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        rapidjson_dumps(rapidjson_loads(data))
    ) == json_loads(data)
    benchmark(rapidjson_loads, data)


def test_loads_github_simplejson(benchmark):
    benchmark.group = "github.json deserialization"
    benchmark.extra_info["lib"] = "simplejson"
    data = read_fixture_str("github.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        simplejson_dumps(simplejson_loads(data))
    ) == json_loads(data)
    benchmark(simplejson_loads, data)


def test_loads_twitter_orjson(benchmark):
    benchmark.group = "twitter.json deserialization"
    benchmark.extra_info["lib"] = "orjson"
    data = read_fixture_str("twitter.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        orjson_dumps(orjson_loads(data))
    ) == json_loads(data)
    benchmark(orjson_loads, data)


def test_loads_twitter_ujson(benchmark):
    benchmark.group = "twitter.json deserialization"
    benchmark.extra_info["lib"] = "ujson"
    data = read_fixture_str("twitter.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        ujson_dumps(ujson_loads(data))
    ) == json_loads(data)
    benchmark(ujson_loads, data)


def test_loads_twitter_json(benchmark):
    benchmark.group = "twitter.json deserialization"
    benchmark.extra_info["lib"] = "json"
    data = read_fixture_str("twitter.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        json_dumps(json_loads(data))
    ) == json_loads(data)
    benchmark(json_loads, data)


def test_loads_twitter_rapidjson(benchmark):
    benchmark.group = "twitter.json deserialization"
    benchmark.extra_info["lib"] = "rapidjson"
    data = read_fixture_str("twitter.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        rapidjson_dumps(rapidjson_loads(data))
    ) == json_loads(data)
    benchmark(rapidjson_loads, data)


def test_loads_twitter_simplejson(benchmark):
    benchmark.group = "twitter.json deserialization"
    benchmark.extra_info["lib"] = "simplejson"
    data = read_fixture_str("twitter.json.xz")
    benchmark.extra_info["correct"] = json_loads(
        simplejson_dumps(simplejson_loads(data))
    ) == json_loads(data)
    benchmark(simplejson_loads, data)
