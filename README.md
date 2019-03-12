# orjson

orjson is a fast, correct JSON library for Python. It benchmarks as the
fastest Python library for JSON and has comprehensive unit, integration, and
interoperability tests.

Its serialization performance is 2x to 3x the nearest
other library and 4.5x to 11.5x the standard library. Its deserialization
performance is 0.95x to 1.1x the nearest other library and 1.2x to 3x
the standard library.

It differs in behavior from other Python JSON libraries in supporting
datetimes, not supporting subclasses without a `default` hook,
serializing UTF-8 to bytes rather than escaped ASCII (e.g., "好" rather than
"\\\u597d") by default, having strict UTF-8 conformance, having strict JSON
conformance on NaN/Infinity/-Infinity, having an option for strict
JSON conformance on 53-bit integers, not supporting pretty
printing, and not supporting all standard library options.

It supports CPython 3.5, 3.6, 3.7, and 3.8. It distributes wheels for Linux,
macOS, and Windows. The repository and issue tracker is
[github.com/ijl/orjson](https://github.com/ijl/orjson).

## Usage

### Install

To install a wheel from PyPI:

```sh
pip install --upgrade orjson
```

To build a release wheel from source, assuming a Rust nightly toolchain
and Python environment:

```sh
git clone https://github.com/ijl/orjson.git && cd orjson
pip install --upgrade pyo3-pack
pyo3-pack build --release --strip --interpreter python3.7
```

There is no runtime dependency other than a manylinux environment (i.e.,
deploying this does not require Rust or non-libc type libraries.)

### Serialize

```python
def dumps(obj: Any, default: Optional[Callable[[Any], Any]], option: Optional[int]) -> bytes: ...
```

`dumps()` serializes Python objects to JSON.

It natively serializes
`str`, `dict`, `list`, `tuple`, `int`, `float`, `datetime.datetime`,
`datetime.date`, `datetime.time`, and `None` instances. It supports
arbitrary types through `default`. It does not serialize subclasses of
supported types natively, but `default` may be used.

It accepts options via an `option` keyword argument. These include:

- `orjson.OPT_STRICT_INTEGER` for enforcing a 53-bit limit on integers. The
limit is otherwise 64 bits, the same as the Python standard library.
- `orjson.OPT_NAIVE_UTC` for assuming `datetime.datetime` objects without a
`tzinfo` are UTC.

To specify multiple options, mask them together, e.g.,
`option=orjson.OPT_STRICT_INTEGER | orjson.OPT_NAIVE_UTC`.

It raises `JSONEncodeError` on an unsupported type. This exception message
describes the invalid object.

It raises `JSONEncodeError` on a `str` that contains invalid UTF-8.

It raises `JSONEncodeError` on an integer that exceeds 64 bits by default or,
with `OPT_STRICT_INTEGER`, 53 bits.

It raises `JSONEncodeError` if a `dict` has a key of a type other than `str`.

It raises `JSONEncodeError` if the output of `default` recurses to handling by
`default` more than five levels deep.

It raises `JSONEncodeError`  if a `tzinfo` on a datetime object is incorrect.

`JSONEncodeError` is a subclass of `TypeError`. This is for compatibility
with the standard library.


```python
import orjson

try:
    val = orjson.dumps(...)
except orjson.JSONEncodeError:
    raise
```

To serialize arbitrary types, specify `default` as a callable that returns
a supported type. `default` may be a function, lambda, or callable class
instance.

```python
>>> import orjson, numpy
>>> def default(obj):
        if isinstance(obj, numpy.ndarray):
            return obj.tolist()
>>> orjson.dumps(numpy.random.rand(2, 2), default=default)
b'[[0.08423896597867486,0.854121264944197],[0.8452845446981371,0.19227780743524303]]'
```

If the `default` callable does not return an object, and an exception
was raised within the `default` function, an exception describing this is
raised. If no object is returned by the `default` callable but also
no exception was raised, it falls through to raising `JSONEncodeError` on an
unsupported type.

The `default` callable may return an object that itself
must be handled by `default` up to five levels deep before an exception
is raised.

### Deserialize

```python
def loads(obj: Union[bytes, str]) -> Union[dict, list, int, float, str, None]: ...
```

`loads()` deserializes JSON to Python objects.

It raises `JSONDecodeError` if given an invalid type or invalid
JSON. This includes if the input contains `NaN`, `Infinity`, or `-Infinity`,
which the standard library allows, but is not valid JSON.

`JSONDecodeError` is a subclass of `ValueError`. This is for
compatibility with the standard library.


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

### datetime

orjson serializes `datetime.datetime` objects to
[RFC 3339](https://tools.ietf.org/html/rfc3339) format, a subset of
ISO 8601.

`datetime.datetime` objects must have `tzinfo` set. For UTC timezones,
`datetime.timezone.utc` is sufficient. For other timezones, `tzinfo`
must be a timezone object from the pendulum, pytz, or dateutil libraries. For
applications in which naive datetimes are known to be UTC, `tzinfo` may be
omitted if `orjson.OPT_NAIVE_UTC` if specified. This does not affect
datetimes with a `tzinfo` set.

```python
>>> import orjson, datetime, pendulum
>>> orjson.dumps(
    datetime.datetime.fromtimestamp(4123518902), option=orjson.OPT_NAIVE_UTC
)
b'"2100-09-01T21:55:02+00:00"'
>>> orjson.dumps(
    datetime.datetime.fromtimestamp(4123518902).replace(tzinfo=datetime.timezone.utc)
)
b'"2100-09-01T21:55:02+00:00"'
>>> orjson.dumps(
    datetime.datetime(2018, 12, 1, 2, 3, 4, 9, tzinfo=pendulum.timezone('Australia/Adelaide'))
)
b'"2018-12-01T02:03:04.9+10:30"'
```

`datetime.time` objects must not have a `tzinfo`. `datetime.date` objects
will always serialize.

```python
>>> import orjson, datetime
>>> orjson.dumps(datetime.date(1900, 1, 2))
b'"1900-01-02"'
>>> orjson.dumps(datetime.time(12, 0, 15, 291290))
b'"12:00:15.291290"'
```

Errors with `tzinfo` result in `JSONEncodeError` being raised.

It is faster to have orjson serialize datetime objects than to do so
before calling `dumps()`. If using an unsupported type such as
`pendulum.datetime`, use `default`.

### int

JSON only requires that implementations accept integers with 53-bit precision.
orjson will, by default, serialize 64-bit integers. This is compatible with
the Python standard library and other non-browser implementations. For
transmitting JSON to a web browser or other strict implementations, `dumps()`
can be configured to raise a `JSONEncodeError` on values exceeding the
53-bit range.

```python
>>> import orjson
>>> orjson.dumps(9007199254740992)
b'9007199254740992'
>>> orjson.dumps(9007199254740992, option=orjson.OPT_STRICT_INTEGER)
JSONEncodeError: Integer exceeds 53-bit max
>>> orjson.dumps(-9007199254740992, option=orjson.OPT_STRICT_INTEGER)
JSONEncodeError: Integer exceeds 53-bit max
```

### UTF-8

orjson raises an exception on invalid UTF-8. This is
necessary because Python 3 str objects may contain UTF-16 surrogates. The
standard library's json module accepts invalid UTF-8.

```python
>>> import orjson, ujson, rapidjson, json
>>> orjson.dumps('\ud800')
JSONEncodeError: str is not valid UTF-8: surrogates not allowed
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
exercising the library's use in web servers (gunicorn using multiprocess/forked
workers) and when
multithreaded. It also uses some tests from the ultrajson library.

## Performance

Serialization and deserialization performance of orjson is better than
ultrajson, rapidjson, simplejson, or json. The benchmarks are done on
fixtures of real data:

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

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.49 |                  2034.9 |                 1    |
| ujson      |                            1.42 |                   703.7 |                 2.89 |
| rapidjson  |                            1.57 |                   638.5 |                 3.19 |
| simplejson |                            2.79 |                   358   |                 5.69 |
| json       |                            2.7  |                   370.3 |                 5.5  |

#### twitter.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            2.4  |                   416.4 |                 1    |
| ujson      |                            2.25 |                   444.4 |                 0.94 |
| rapidjson  |                            3.04 |                   328.4 |                 1.27 |
| simplejson |                            2.49 |                   401.8 |                 1.04 |
| json       |                            2.82 |                   354.3 |                 1.17 |

#### github.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.05 |                 18299.4 |                 1    |
| ujson      |                            0.14 |                  7126.2 |                 2.57 |
| rapidjson  |                            0.16 |                  6370.2 |                 2.87 |
| simplejson |                            0.31 |                  3274.2 |                 5.59 |
| json       |                            0.27 |                  3713   |                 4.93 |

#### github.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.21 |                  4873.7 |                 1    |
| ujson      |                            0.22 |                  4493.4 |                 1.08 |
| rapidjson  |                            0.27 |                  3721.3 |                 1.31 |
| simplejson |                            0.22 |                  4593.9 |                 1.06 |
| json       |                            0.24 |                  4142.5 |                 1.18 |

#### citm_catalog.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.81 |                  1223.7 |                 1    |
| ujson      |                            2.66 |                   376   |                 3.27 |
| rapidjson  |                            2.45 |                   408.7 |                 3.01 |
| simplejson |                           11    |                    91   |                13.53 |
| json       |                            6.03 |                   165.8 |                 7.42 |

#### citm_catalog.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            4.6  |                   217   |                 1    |
| ujson      |                            4.49 |                   222.8 |                 0.98 |
| rapidjson  |                            5.68 |                   175.8 |                 1.24 |
| simplejson |                            6.06 |                   165.2 |                 1.32 |
| json       |                            5.99 |                   166.8 |                 1.3  |

#### canada.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            4.29 |                   232.4 |                 1    |
| ujson      |                            8.71 |                   115.7 |                 2.03 |
| rapidjson  |                           43.69 |                    22.9 |                10.18 |
| simplejson |                           66.56 |                    15   |                15.51 |
| json       |                           49.27 |                    19.9 |                11.48 |

#### canada.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            8.63 |                   116.4 |                 1    |
| ujson      |                            8.55 |                   116.9 |                 0.99 |
| rapidjson  |                           29.37 |                    34.1 |                 3.4  |
| simplejson |                           26.56 |                    37.6 |                 3.08 |
| json       |                           27.56 |                    36.3 |                 3.19 |


This was measured using Python 3.7.2 on Linux with orjson 2.0.0, ujson 1.35,
python-rapidson 0.6.3, and simplejson 3.16.0.

The results can be reproduced using the `pybench` and `graph` scripts.

## License

orjson is dual licensed under the Apache 2.0 and MIT licenses. It contains
tests from the hyperjson and ultrajson libraries. It is implemented using
the [serde_json](https://github.com/serde-rs/json) and
[pyo3](https://github.com/PyO3/pyo3) libraries.
