# SPDX-License-Identifier: (Apache-2.0 OR MIT)

from json import dumps as _json_dumps
from json import loads as json_loads

from rapidjson import dumps as _rapidjson_dumps
from rapidjson import loads as rapidjson_loads
from simplejson import dumps as _simplejson_dumps
from simplejson import loads as simplejson_loads
from ujson import dumps as _ujson_dumps
from ujson import loads as ujson_loads

from orjson import dumps as _orjson_dumps
from orjson import loads as orjson_loads

# dumps wrappers that return UTF-8


def orjson_dumps(obj):
    return _orjson_dumps(obj)


def ujson_dumps(obj):
    return _ujson_dumps(obj).encode("utf-8")


def rapidjson_dumps(obj):
    return _rapidjson_dumps(obj).encode("utf-8")


def json_dumps(obj):
    return _json_dumps(obj).encode("utf-8")


def simplejson_dumps(obj):
    return _simplejson_dumps(obj).encode("utf-8")


# Add new libraries here (pair of UTF-8 dumper, loader)
libraries = {
    "orjson": (orjson_dumps, orjson_loads),
    "ujson": (ujson_dumps, ujson_loads),
    "json": (json_dumps, json_loads),
    "rapidjson": (rapidjson_dumps, rapidjson_loads),
    "simplejson": (simplejson_dumps, simplejson_loads),
}

# Add new JSON files here (corresponding to ../data/*.json.xz)
fixtures = [
    "canada.json",
    "citm_catalog.json",
    "github.json",
    "twitter.json",
]
