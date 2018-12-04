# orjson

orjson is a fast JSON library for Python. It benchmarks as the fastest Python
library for JSON serialization, about twice as fast or more as the nearest
other library, with deserialization performance slightly worse to similar to
the fastest library.

It supports CPython 3.5, 3.6, and 3.7. It is not intended
as a drop-in replacement for the standard library's json module.

## Usage

### Install

To install a manylinux wheel from PyPI:

```sh
pip install --upgrade orjson
```

To build a release wheel from source, assuming a Rust nightly toolchain
and Python environment:

```sh
git checkout https://github.com/ijl/orjson.git && cd orjson
git submodule init && git submodule update
pip install --upgrade pyo3-pack
pyo3-pack build --release --strip --interpreter python3.7
```

There is no runtime dependency other than a manylinux environment (i.e.,
deploying this does not require Rust or non-libc type libraries.)

### Serialize

```python
def dumps(obj: Union[str, bytes, dict, list, tuple, int, float, None]) -> bytes: ...
```

`dumps()` serializes Python objects to JSON.

It has no options, does not support hooks for custom objects, and does not
support subclasses.

It raises `TypeError` on an unsupported type or a number that is too large.
The error message describes the invalid object.

```python
import orjson

try:
    val = orjson.dumps(...)
except TypeError:
    raise
```

### Deserialize

```python
def loads(obj: Union[bytes, str]) -> Union[dict, list, int, float, str]: ...
```

`loads()` deserializes JSON to Python objects.

It raises `orjson.JSONDecodeError` on invalid input.


```python
import orjson

try:
    val = orjson.loads(...)
except orjson.JSONDecodeError:
    raise
```

### Comparison

There are slight differences in output between libraries. The differences
are not an issue for interoperability. Note orjson returns bytes. Its output
is slightly smaller as well.

```python
>>> import orjson, ujson, rapidjson, json
>>> data = {'bool': True, '🐈':'哈哈', 'int': 9223372036854775807, 'float': 1.337e+40}
>>> orjson.dumps(data)
b'{"bool":true,"\xf0\x9f\x90\x88":"\xe5\x93\x88\xe5\x93\x88","int":9223372036854775807,"float":1.337e40}'
>>> ujson.dumps(data)
'{"bool":true,"\\ud83d\\udc08":"\\u54c8\\u54c8","int":9223372036854775807,"float":1.337000000000000e+40}'
>>> rapidjson.dumps(data)
'{"bool":true,"\\uD83D\\uDC08":"\\u54C8\\u54C8","int":9223372036854775807,"float":1.337e+40}'
>>> json.dumps(data)
'{"bool": true, "\\ud83d\\udc08": "\\u54c8\\u54c8", "int": 9223372036854775807, "float": 1.337e+40}'
```

## Testing

The library has comprehensive tests. There are unit tests against the
roundtrip, jsonchecker, and fixtures files of the
[nativejson-benchmark](https://github.com/miloyip/nativejson-benchmark)
repository. It is tested to not crash against the
[Big List of Naughty Strings](https://github.com/minimaxir/big-list-of-naughty-strings).
There are integration tests exercising the library's use in web
servers (uwsgi and gunicorn, using multiprocess/forked workers) and when
multithreaded. It also uses some tests from the ultrajson library.

## Performance

Serialization performance of orjson is better than ultrajson, rapidjson, or
json. Deserialization performance is worse to about the same as ultrajson.

![alt text](doc/twitter-serialize.png "twitter.json serialization")
![alt text](doc/citm_catalog-serialize.png "citm_catalog.json serialization")
![alt text](doc/canada-serialize.png "canada.json serialization")

#### canada.json deserialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    8.72 |                   114.8 |                 1.25 |
| ujson     |                    6.95 |                   138.4 |                 1    |
| rapidjson |                   27.75 |                    36   |                 3.99 |
| json      |                   27.22 |                    36.6 |                 3.92 |

#### canada.json serialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    5.12 |                   195.4 |                 1    |
| ujson     |                    8.19 |                   122.1 |                 1.6  |
| rapidjson |                   44.48 |                    22.5 |                 8.69 |
| json      |                   46.85 |                    21.3 |                 9.16 |

#### citm_catalog.json deserialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    6.06 |                   164.9 |                 1.12 |
| ujson     |                    5.4  |                   185.2 |                 1    |
| rapidjson |                    7.26 |                   137.6 |                 1.35 |
| json      |                    7.49 |                   132.2 |                 1.39 |

#### citm_catalog.json serialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    1.02 |                   980.6 |                 1    |
| ujson     |                    2.53 |                   394.4 |                 2.49 |
| rapidjson |                    2.37 |                   421.9 |                 2.33 |
| json      |                    5.32 |                   188   |                 5.22 |

#### twitter.json deserialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    2.98 |                   335.3 |                 1.25 |
| ujson     |                    2.39 |                   419.3 |                 1    |
| rapidjson |                    3.12 |                   318.8 |                 1.31 |
| json      |                    3.12 |                   318.8 |                 1.31 |

#### twitter.json serialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    0.55 |                  1815.1 |                 1    |
| ujson     |                    1.46 |                   684.9 |                 2.65 |
| rapidjson |                    1.55 |                   643.9 |                 2.82 |
| json      |                    2.18 |                   458.7 |                 3.95 |

This was measured using orjson 1.0.0 on Python 3.7.1. The above can be
reproduced using the `pybench` and `graph` scripts.

## License

orjson is dual licensed under the Apache 2.0 and MIT licenses. It contains
code from the hyperjson and ultrajson libraries. It is implemented using
the [serde_json](https://github.com/serde-rs/json) and
[pyo3](https://github.com/PyO3/pyo3) libraries.
