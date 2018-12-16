# orjson

orjson is a fast JSON library for Python. It benchmarks as the fastest Python
library for JSON serialization, with 1.6x to 2.6x the performance as the nearest
other library, with deserialization performance of 0.95x to 1.2x
the nearest other library.

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
git clone --recurse-submodules https://github.com/ijl/orjson.git && cd orjson
virtualenv .venv && source .venv/bin/activate
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

It raises `orjson.JSONDecodeError` on invalid input. This exception is a
subclass of `ValueError`.


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
json. Deserialization performance is better to about the same as ultrajson.

![alt text](doc/twitter_serialization.png "twitter.json serialization")
![alt text](doc/twitter_deserialization.png "twitter.json deserialization")
![alt text](doc/citm_catalog_serialization.png "citm_catalog.json serialization")
![alt text](doc/citm_catalog_deserialization.png "citm_catalog.json deserialization")
![alt text](doc/github_serialization.png "github.json serialization")
![alt text](doc/github_deserialization.png "github.json deserialization")
![alt text](doc/canada_serialization.png "canada.json serialization")
![alt text](doc/canada_deserialization.png "canada.json deserialization")

#### canada.json deserialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    7.59 |                   131.8 |                 1    |
| ujson     |                    7.26 |                   133.5 |                 0.96 |
| rapidjson |                   26.72 |                    37.4 |                 3.52 |
| json      |                   26.78 |                    37.3 |                 3.53 |

#### canada.json serialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    4.99 |                   200.3 |                 1    |
| ujson     |                    8.16 |                   122.5 |                 1.64 |
| rapidjson |                   43.27 |                    23.1 |                 8.67 |
| json      |                   48.15 |                    20.8 |                 9.65 |

#### citm_catalog.json deserialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    5.05 |                   198.2 |                 1    |
| ujson     |                    6.2  |                   161.2 |                 1.23 |
| rapidjson |                    6.57 |                   152.2 |                 1.3  |
| json      |                    6.62 |                   151.1 |                 1.31 |

#### citm_catalog.json serialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    1    |                   997.4 |                 1    |
| ujson     |                    2.54 |                   394.1 |                 2.53 |
| rapidjson |                    2.38 |                   419.5 |                 2.38 |
| json      |                    5.26 |                   190   |                 5.25 |

#### github.json deserialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    0.23 |                  4310.6 |                 1    |
| ujson     |                    0.23 |                  4414.3 |                 0.98 |
| rapidjson |                    0.23 |                  4229.4 |                 1    |
| json      |                    0.23 |                  4176.3 |                 1    |

#### github.json serialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    0.06 |                 16357.9 |                 1    |
| ujson     |                    0.13 |                  7531.2 |                 2.17 |
| rapidjson |                    0.16 |                  6362.9 |                 2.57 |
| json      |                    0.23 |                  4242.5 |                 3.8  |

#### twitter.json deserialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    2.6  |                   385.5 |                 1    |
| ujson     |                    2.98 |                   336.5 |                 1.15 |
| rapidjson |                    2.84 |                   339.1 |                 1.09 |
| json      |                    2.84 |                   345.9 |                 1.09 |

#### twitter.json serialization

| Library   |   Median (milliseconds) |   Operations per second |   Relative (latency) |
|-----------|-------------------------|-------------------------|----------------------|
| orjson    |                    0.56 |                  1790   |                 1    |
| ujson     |                    1.44 |                   693.9 |                 2.58 |
| rapidjson |                    1.57 |                   636.1 |                 2.82 |
| json      |                    2.21 |                   452   |                 3.96 |

This was measured using orjson 1.2.0 on Python 3.7.1 and Linux. The above can be
reproduced using the `pybench` and `graph` scripts.

## License

orjson is dual licensed under the Apache 2.0 and MIT licenses. It contains
code from the hyperjson and ultrajson libraries. It is implemented using
the [serde_json](https://github.com/serde-rs/json) and
[pyo3](https://github.com/PyO3/pyo3) libraries.
