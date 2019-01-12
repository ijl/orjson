# orjson

orjson is a fast JSON library for Python. It benchmarks as the fastest Python
library for JSON. Its serialization performance is 2x to 3x the nearest
other library and 4.5x to 11.5x the standard library. Its deserialization
performance is 1.05x to 1.2x the nearest other library and 1.2x to 4x
the standard library.

It supports CPython 3.5, 3.6, and 3.7. Its API is a subset of the
API of the standard library's json module.

## Usage

### Install

To install a manylinux wheel from PyPI:

```sh
pip install --upgrade orjson
```

To build a release wheel from source, assuming a Rust nightly toolchain
and Python environment:

```sh
git clone --recurse-submodules https://github.com/ijl/orjson.git && cd orjson
pip install --upgrade pyo3-pack
pyo3-pack build --release --strip --interpreter python3.7
```

There is no runtime dependency other than a manylinux environment (i.e.,
deploying this does not require Rust or non-libc type libraries.)

### Serialize

```python
def dumps(obj: Union[str, dict, list, tuple, int, float, None]) -> bytes: ...
```

`dumps()` serializes Python objects to JSON.

It has no options, does not support hooks for custom objects, and does not
support subclasses.

It raises `TypeError` on an unsupported type. This exception message
describes the invalid object.

It raises `TypeError` on a `str` that contains invalid UTF-8.

It raises `TypeError` on an integer that exceeds 64 bits. This is the same
as the standard library's `json` module.

It raises `TypeError` if a `dict` has a key of a type other than `str`.


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

It raises `orjson.JSONDecodeError` if given an invalid type or invalid
JSON. This exception is a subclass of `ValueError`.


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

### UTF-8

orjson raises an exception on invalid UTF-8. This is
necessary because Python 3 str objects may contain UTF-16 surrogates. The
standard library's json module accepts invalid UTF-8.

```python
>>> import orjson, ujson, rapidjson, json
>>> orjson.dumps('\ud800')
TypeError: str is not valid UTF-8: surrogates not allowed
>>> ujson.dumps('\ud800')
UnicodeEncodeError: 'utf-8' codec ...
>>> rapidjson.dumps('\ud800')
UnicodeEncodeError: 'utf-8' codec ...
>>> json.dumps('\ud800')
'"\\ud800"'
```

```python
>>> import orjson, ujson, rapidjson, json
>>> orjson.loads('"\\ud800"')
JSONDecodeError: unexpected end of hex escape at line 1 column 8: line 1 column 1 (char 0)
>>> ujson.loads('"\\ud800"')
''
>>> rapidjson.loads('"\\ud800"')
ValueError: Parse error at offset 1: The surrogate pair in string is invalid.
>>> json.loads('"\\ud800"')
'\ud800'
```

## Testing

The library has comprehensive tests. There are unit tests against the
roundtrip, jsonchecker, and fixtures files of the
[nativejson-benchmark](https://github.com/miloyip/nativejson-benchmark)
repository. It is tested to not crash against the
[Big List of Naughty Strings](https://github.com/minimaxir/big-list-of-naughty-strings).
It is tested to not leak memory. It is tested to be correct against
input from the PyJFuzz JSON fuzzer. It is tested to not crash
against and not accept invalid UTF-8. There are integration tests
exercising the library's use in web servers (uwsgi and gunicorn,
using multiprocess/forked workers) and when
multithreaded. It also uses some tests from the ultrajson library.

## Performance

Serialization and deserialization performance of orjson is better than
ultrajson, rapidjson, or json. The benchmarks are done on fixtures of real data:

* twitter.json, 631.5KiB, results of a search on Twitter for "一", containing
CJK strings, dictionaries of strings and arrays of dictionaries, indented.

* github.json, 55.8KiB, a GitHub activity feed, containing dictionaries of
strings and arrays of dictionaries, not indented.

* citm_catalog.json, 1.7MiB, concert data, containing nested dictionaries of
strings and arrays of integers, indented.

* canada.json, 2.2MiB, coordinates of the Canadian border in GeoJSON
format, containing floats and arrays, indented.

![alt text](doc/twitter_serialization.png "twitter.json serialization")
![alt text](doc/twitter_deserialization.png "twitter.json deserialization")
![alt text](doc/github_serialization.png "github.json serialization")
![alt text](doc/github_deserialization.png "github.json deserialization")
![alt text](doc/citm_catalog_serialization.png "citm_catalog.json serialization")
![alt text](doc/citm_catalog_deserialization.png "citm_catalog.json deserialization")
![alt text](doc/canada_serialization.png "canada.json serialization")
![alt text](doc/canada_deserialization.png "canada.json deserialization")

#### twitter.json serialization

| Library   |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|---------------------------------|-------------------------|----------------------|
| orjson    |                            0.48 |                  2077.6 |                 1    |
| ujson     |                            1.48 |                   664.6 |                 3.09 |
| rapidjson |                            1.59 |                   626.5 |                 3.32 |
| json      |                            2.24 |                   443.9 |                 4.68 |

#### twitter.json deserialization

| Library   |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|---------------------------------|-------------------------|----------------------|
| orjson    |                            2.38 |                   418.8 |                 1    |
| ujson     |                            2.67 |                   373   |                 1.12 |
| rapidjson |                            2.78 |                   359.5 |                 1.16 |
| json      |                            2.77 |                   359.7 |                 1.16 |

#### github.json serialization

| Library   |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|---------------------------------|-------------------------|----------------------|
| orjson    |                            0.06 |                 17745   |                 1    |
| ujson     |                            0.14 |                  7107.1 |                 2.49 |
| rapidjson |                            0.16 |                  6253.9 |                 2.86 |
| json      |                            0.25 |                  3972.5 |                 4.49 |

#### github.json deserialization

| Library   |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|---------------------------------|-------------------------|----------------------|
| orjson    |                            0.2  |                  4929.7 |                 1    |
| ujson     |                            0.22 |                  4605.2 |                 1.08 |
| rapidjson |                            0.24 |                  4166.5 |                 1.19 |
| json      |                            0.24 |                  4150.8 |                 1.19 |

#### citm_catalog.json serialization

| Library   |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|---------------------------------|-------------------------|----------------------|
| orjson    |                            0.76 |                  1302   |                 1    |
| ujson     |                            2.58 |                   387.2 |                 3.38 |
| rapidjson |                            2.37 |                   421.1 |                 3.11 |
| json      |                            5.41 |                   184.4 |                 7.09 |

#### citm_catalog.json deserialization

| Library   |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|---------------------------------|-------------------------|----------------------|
| orjson    |                            4.28 |                   233.1 |                 1    |
| ujson     |                            5.06 |                   197.2 |                 1.18 |
| rapidjson |                            5.82 |                   171.7 |                 1.36 |
| json      |                            5.81 |                   171.8 |                 1.36 |

#### canada.json serialization

| Library   |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|---------------------------------|-------------------------|----------------------|
| orjson    |                            4.04 |                   247.7 |                 1    |
| ujson     |                            8.43 |                   118.6 |                 2.09 |
| rapidjson |                           43.93 |                    22.7 |                10.88 |
| json      |                           47.23 |                    21.1 |                11.7  |

#### canada.json deserialization

| Library   |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|---------------------------------|-------------------------|----------------------|
| orjson    |                            6.69 |                   147.6 |                 1    |
| ujson     |                            7.17 |                   139.4 |                 1.07 |
| rapidjson |                           26.77 |                    37.4 |                 4    |
| json      |                           26.59 |                    37.6 |                 3.97 |


This was measured using orjson 1.3.0 on Python 3.7.2 and Linux.

The results can be reproduced using the `pybench` and `graph` scripts.

## License

orjson is dual licensed under the Apache 2.0 and MIT licenses. It contains
tests from the hyperjson and ultrajson libraries. It is implemented using
the [serde_json](https://github.com/serde-rs/json) and
[pyo3](https://github.com/PyO3/pyo3) libraries.
