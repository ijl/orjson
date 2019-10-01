# orjson

orjson is a fast, correct JSON library for Python. It benchmarks as the
fastest Python library for JSON and is more correct than the standard json
library or third-party libraries.

Its serialization performance is 2.5x to 9.5x the nearest
other library and 4x to 12x the standard library. Its deserialization
performance is 1.2x to 1.3x the nearest other library and 1.4x to 2x
the standard library.

It differs in behavior from other Python JSON libraries in supporting
datetimes, not supporting subclasses without a `default` hook,
serializing UTF-8 to bytes rather than escaped ASCII (e.g., "好" rather than
"\\\u597d") by default, having strict UTF-8 conformance, having strict JSON
conformance on NaN/Infinity/-Infinity, having an option for strict
JSON conformance on 53-bit integers, not supporting pretty
printing, and not supporting all standard library options.

orjson supports CPython 3.5, 3.6, 3.7, and 3.8. It distributes wheels for Linux,
macOS, and Windows. The manylinux1 wheel differs from PEP 513 in requiring
glibc 2.18, released 2013, or later. orjson does not currently support PyPy.

orjson is licensed under both the Apache 2.0 and MIT licenses. The
repository and issue tracker is
[github.com/ijl/orjson](https://github.com/ijl/orjson), and patches may be
submitted there.

## Usage

### Install

To install a wheel from PyPI:

```sh
pip install --upgrade orjson
```

To build from source requires [Rust](https://www.rust-lang.org/) on the
`nightly` channel. Package a wheel from a PEP 517 source distribution using
pip:

```sh
pip wheel --no-binary=orjson orjson
```

There are no runtime dependencies other than libc. orjson is compatible with
systems using glibc earlier than 2.18 if compiled on such a system. Tooling
does not currently support musl libc.

### Serialize

```python
def dumps(__obj: Any, default: Optional[Callable[[Any], Any]] = ..., option: Optional[int] = ...) -> bytes: ...
```

`dumps()` serializes Python objects to JSON.

It natively serializes
`str`, `dict`, `list`, `tuple`, `int`, `float`, `bool`,
`typing.TypedDict`, `datetime.datetime`,
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

It raises `JSONEncodeError` on circular references.

It raises `JSONEncodeError`  if a `tzinfo` on a datetime object is incorrect.

`JSONEncodeError` is a subclass of `TypeError`. This is for compatibility
with the standard library.

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
def loads(__obj: Union[bytes, str]) -> Any: ...
```

`loads()` deserializes JSON to Python objects. It deserializes to `dict`,
`list`, `int`, `float`, `str`, `bool`, and `None` objects.

Either `bytes` or `str` input are accepted. If the input exists as
`bytes` (was read directly from a source), it is recommended to
pass `bytes`. This has lower memory usage and lower latency.

orjson maintains a cache of map keys for the duration of the process. This
causes a net reduction in memory usage by avoiding duplicate strings. The
keys must be at most 64 chars to be cached and 512 entries are stored.

It raises `JSONDecodeError` if given an invalid type or invalid
JSON. This includes if the input contains `NaN`, `Infinity`, or `-Infinity`,
which the standard library allows, but is not valid JSON.

`JSONDecodeError` is a subclass of `json.JSONDecodeError` and `ValueError`.
This is for compatibility with the standard library.

### datetime

orjson serializes `datetime.datetime` objects to
[RFC 3339](https://tools.ietf.org/html/rfc3339) format, a subset of
ISO 8601.

`datetime.datetime` objects serialize with or without a `tzinfo`. For a full
 RFC 3339 representation, `tzinfo` must be present or `orjson.OPT_NAIVE_UTC`
 must be specified (e.g., for timestamps stored in a database in UTC and
 deserialized by the database adapter without a `tzinfo`). If a
 `tzinfo` is not present, a timezone offset is not serialized.

`tzinfo`, if specified, must be a timezone object that is either
`datetime.timezone.utc` or from the `pendulum`, `pytz`, or
`dateutil`/`arrow` libraries.

```python
>>> import orjson, datetime, pendulum
>>> orjson.dumps(
    datetime.datetime(2018, 12, 1, 2, 3, 4, 9, tzinfo=pendulum.timezone('Australia/Adelaide'))
)
b'"2018-12-01T02:03:04.000009+10:30"'
>>> orjson.dumps(
    datetime.datetime.fromtimestamp(4123518902).replace(tzinfo=datetime.timezone.utc)
)
b'"2100-09-01T21:55:02+00:00"'
>>> orjson.dumps(
    datetime.datetime.fromtimestamp(4123518902)
)
b'"2100-09-01T21:55:02"'
```

`orjson.OPT_NAIVE_UTC`, if specified, only applies to objects that do not have
a `tzinfo`.

```python
>>> import orjson, datetime, pendulum
>>> orjson.dumps(
    datetime.datetime.fromtimestamp(4123518902),
    option=orjson.OPT_NAIVE_UTC
)
b'"2100-09-01T21:55:02+00:00"'
>>> orjson.dumps(
    datetime.datetime(2018, 12, 1, 2, 3, 4, 9, tzinfo=pendulum.timezone('Australia/Adelaide')),
    option=orjson.OPT_NAIVE_UTC
)
b'"2018-12-01T02:03:04.000009+10:30"'
```

`datetime.time` objects must not have a `tzinfo`.

```python
>>> import orjson, datetime
>>> orjson.dumps(datetime.time(12, 0, 15, 290))
b'"12:00:15.000290"'
```

`datetime.date` objects will always serialize.

```python
>>> import orjson, datetime
>>> orjson.dumps(datetime.date(1900, 1, 2))
b'"1900-01-02"'
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
JSONEncodeError: Integer exceeds 53-bit range
>>> orjson.dumps(-9007199254740992, option=orjson.OPT_STRICT_INTEGER)
JSONEncodeError: Integer exceeds 53-bit range
```

### float

orjson serializes and deserializes floats with no loss of precision and
consistent rounding. The same behavior is observed in rapidjson, simplejson,
and json. ujson is inaccurate in both serialization and deserialization,
i.e., it modifies the data.

`orjson.dumps()` serializes Nan, Infinity, and -Infinity, which are not
compliant JSON, as `null`:

```python
>>> import orjson
>>> orjson.dumps([float("NaN"), float("Infinity"), float("-Infinity")])
b'[null,null,null]'
>>> ujson.dumps([float("NaN"), float("Infinity"), float("-Infinity")])
OverflowError: Invalid Inf value when encoding double
>>> rapidjson.dumps([float("NaN"), float("Infinity"), float("-Infinity")])
'[NaN,Infinity,-Infinity]'
>>> json.dumps([float("NaN"), float("Infinity"), float("-Infinity")])
'[NaN, Infinity, -Infinity]'
```

### UTF-8

orjson raises an exception on invalid UTF-8. This is
necessary because Python 3 `str` objects may contain UTF-16 surrogates.

The standard library's json module deserializes and serializes invalid UTF-8.

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

The library has comprehensive tests. There are tests against fixtures in the
[JSONTestSuite](https://github.com/nst/JSONTestSuite) and
[nativejson-benchmark](https://github.com/miloyip/nativejson-benchmark)
repositories. It is tested to not crash against the
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

### Latency

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
| orjson     |                            0.76 |                  1308.1 |                 1    |
| ujson      |                            2.14 |                   468   |                 2.8  |
| rapidjson  |                            2.14 |                   467.5 |                 2.8  |
| simplejson |                            3.45 |                   289.1 |                 4.52 |
| json       |                            3.6  |                   277.9 |                 4.71 |

#### twitter.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            2.8  |                   357.9 |                 1    |
| ujson      |                            3.37 |                   296.1 |                 1.21 |
| rapidjson  |                            4.48 |                   222.9 |                 1.6  |
| simplejson |                            3.71 |                   269.3 |                 1.33 |
| json       |                            4.13 |                   242.3 |                 1.48 |

#### github.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.08 |                 12430.9 |                 1    |
| ujson      |                            0.21 |                  4774.2 |                 2.6  |
| rapidjson  |                            0.23 |                  4350.4 |                 2.87 |
| simplejson |                            0.44 |                  2290.5 |                 5.42 |
| json       |                            0.36 |                  2786.8 |                 4.46 |

#### github.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.25 |                  3939.5 |                 1    |
| ujson      |                            0.33 |                  3053.2 |                 1.29 |
| rapidjson  |                            0.39 |                  2589.7 |                 1.52 |
| simplejson |                            0.28 |                  3549.6 |                 1.11 |
| json       |                            0.36 |                  2767.7 |                 1.43 |

#### citm_catalog.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            1.17 |                   856   |                 1    |
| ujson      |                            3.65 |                   273   |                 3.11 |
| rapidjson  |                            3.45 |                   274.1 |                 2.94 |
| simplejson |                           14.67 |                    68.2 |                12.5  |
| json       |                            8.19 |                   122.2 |                 6.98 |

#### citm_catalog.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            5.62 |                   178   |                 1    |
| ujson      |                            6.76 |                   148.3 |                 1.2  |
| rapidjson  |                            7.65 |                   129   |                 1.36 |
| simplejson |                            9.05 |                   110.4 |                 1.61 |
| json       |                            8.24 |                   120.8 |                 1.47 |

#### canada.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            7.58 |                   138.6 |                 1    |
| ujson      |                                 |                         |                      |
| rapidjson  |                           73.02 |                    13.7 |                 9.64 |
| simplejson |                          101.21 |                     9.8 |                13.36 |
| json       |                           91.45 |                    11.5 |                12.07 |

#### canada.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                           22.89 |                    43.8 |                 1    |
| ujson      |                                 |                         |                      |
| rapidjson  |                           47.18 |                    21.3 |                 2.06 |
| simplejson |                           44.41 |                    22.5 |                 1.94 |
| json       |                           43.72 |                    22.9 |                 1.91 |

If a row is blank, the library did not serialize and deserialize the fixture without
modifying it, e.g., returning different values for floating point numbers.

### Memory

orjson's memory usage when deserializing is similar to or lower than
the standard library and other third-party libraries.

This measures, in the first column, RSS after importing a library and reading
the fixture, and in the second column, increases in RSS after repeatedly
calling `loads()` on the fixture.

#### twitter.json

| Library    |   import, read() RSS (MiB) |   loads() increase in RSS (MiB) |
|------------|----------------------------|---------------------------------|
| orjson     |                       12.8 |                             2.7 |
| ujson      |                       12.8 |                             4.6 |
| rapidjson  |                       14.3 |                             6.4 |
| simplejson |                       13   |                             2.8 |
| json       |                       12.3 |                             2.5 |

#### github.json

| Library    |   import, read() RSS (MiB) |   loads() increase in RSS (MiB) |
|------------|----------------------------|---------------------------------|
| orjson     |                       12.3 |                             0.3 |
| ujson      |                       12.4 |                             0.5 |
| rapidjson  |                       13.8 |                             0.5 |
| simplejson |                       12.4 |                             0.3 |
| json       |                       11.7 |                             0.3 |

#### citm_catalog.json

| Library    |   import, read() RSS (MiB) |   loads() increase in RSS (MiB) |
|------------|----------------------------|---------------------------------|
| orjson     |                       13.9 |                             8.3 |
| ujson      |                       13.8 |                            12   |
| rapidjson  |                       15.5 |                            20.3 |
| simplejson |                       14.1 |                            21.8 |
| json       |                       13.4 |                            20.2 |

#### canada.json

| Library    |   import, read() RSS (MiB) |   loads() increase in RSS (MiB) |
|------------|----------------------------|---------------------------------|
| orjson     |                       16.5 |                            17.5 |
| ujson      |                            |                                 |
| rapidjson  |                       17.8 |                            19.8 |
| simplejson |                       16.5 |                            21.3 |
| json       |                       16.1 |                            21.3 |

### Reproducing

The above was measured using Python 3.7.4 on Linux with orjson 2.0.10,
ujson 1.35, python-rapidson 0.8.0, and simplejson 3.16.0.

The latency results can be reproduced using the `pybench` and `graph`
scripts. The memory results can be reproduced using the `pymem` script.
