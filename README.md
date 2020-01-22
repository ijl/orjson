# orjson

orjson is a fast, correct JSON library for Python. It
[benchmarks](https://github.com/ijl/orjson#performance) as the fastest Python
library for JSON and is more correct than the standard json library or
third-party libraries. It serializes
[dataclass](https://github.com/ijl/orjson#dataclass) and
[datetime](https://github.com/ijl/orjson#datetime) instances.

Its serialization performance on fixtures of real data is 2.5x to 9.5x the
nearest other library and 4x to 12x the standard library. Its deserialization
performance on the same fixtures is 1.2x to 1.3x the nearest other
library and 1.4x to 2x the standard library.

Its features and drawbacks compared to other Python JSON libraries:

* serializes `dataclass` instances 30x faster than other libraries
* serializes `datetime`, `date`, and `time` instances to RFC 3339 format,
e.g., "1970-01-01T00:00:00+00:00"
* serializes to `bytes` rather than `str`, i.e., is not a drop-in replacement
* serializes `str` without escaping unicode to ASCII, e.g., "好" rather than
"\\\u597d"
* serializes `float` 10x faster and deserializes twice as fast as other
libraries
* serializes arbitrary types using a `default` hook
* has strict UTF-8 conformance, more correct than the standard library
* has strict JSON conformance in not supporting Nan/Infinity/-Infinity
* has an option for strict JSON conformance on 53-bit integers with default
support for 64-bit
* does not support subclasses by default, requiring use of `default` hook
* does not support pretty printing
* does not support sorting `dict` by keys
* does not provide `load()` or `dump()` functions for reading from/writing to
file-like objects

orjson supports CPython 3.6, 3.7, 3.8, and 3.9. It distributes wheels for
Linux, macOS, and Windows. The manylinux1 wheel differs from PEP 513 in
requiring glibc 2.18, released 2013, or later. orjson does not support PyPy.

orjson is licensed under both the Apache 2.0 and MIT licenses. The
repository and issue tracker is
[github.com/ijl/orjson](https://github.com/ijl/orjson), and patches may be
submitted there. There is a
[CHANGELOG](https://github.com/ijl/orjson/blob/master/CHANGELOG.md)
available in the repository.

1. [Usage](https://github.com/ijl/orjson#usage)
    1. [Install](https://github.com/ijl/orjson#install)
    2. [Serialize](https://github.com/ijl/orjson#serialize)
        1. [default](https://github.com/ijl/orjson#default)
        2. [option](https://github.com/ijl/orjson#option)
    3. [Deserialize](https://github.com/ijl/orjson#deserialize)
2. [Types](https://github.com/ijl/orjson#types)
    1. [dataclass](https://github.com/ijl/orjson#dataclass)
    2. [datetime](https://github.com/ijl/orjson#datetime)
    3. [float](https://github.com/ijl/orjson#float)
    4. [int](https://github.com/ijl/orjson#int)
    5. [str](https://github.com/ijl/orjson#str)
    6. [UUID](https://github.com/ijl/orjson#UUID)
3. [Testing](https://github.com/ijl/orjson#testing)
4. [Performance](https://github.com/ijl/orjson#performance)
    1. [Latency](https://github.com/ijl/orjson#latency)
    2. [Memory](https://github.com/ijl/orjson#memory)
    3. [Reproducing](https://github.com/ijl/orjson#reproducing)
5. [License](https://github.com/ijl/orjson#license)

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
def dumps(
    __obj: Any,
    default: Optional[Callable[[Any], Any]] = ...,
    option: Optional[int] = ...,
) -> bytes: ...
```

`dumps()` serializes Python objects to JSON.

It natively serializes
`str`, `dict`, `list`, `tuple`, `int`, `float`, `bool`,
`dataclasses.dataclass`, `typing.TypedDict`, `datetime.datetime`,
`datetime.date`, `datetime.time`, and `None` instances. It supports
arbitrary types through `default`. It does not serialize subclasses of
supported types natively, with the exception of `dataclasses.dataclass`
subclasses.

It raises `JSONEncodeError` on an unsupported type. This exception message
describes the invalid object.

It raises `JSONEncodeError` on a `str` that contains invalid UTF-8.

It raises `JSONEncodeError` on an integer that exceeds 64 bits by default or,
with `OPT_STRICT_INTEGER`, 53 bits.

It raises `JSONEncodeError` if a `dict` has a key of a type other than `str`.

It raises `JSONEncodeError` if the output of `default` recurses to handling by
`default` more than 254 levels deep.

It raises `JSONEncodeError` on circular references.

It raises `JSONEncodeError`  if a `tzinfo` on a datetime object is incorrect.

`JSONEncodeError` is a subclass of `TypeError`. This is for compatibility
with the standard library.

#### default

To serialize a subclass or arbitrary types, specify `default` as a
callable that returns a supported type. `default` may be a function,
lambda, or callable class instance.

```python
>>> import orjson, numpy
>>>
def default(obj):
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
must be handled by `default` up to 254 times before an exception
is raised.

#### option

To modify how data is serialized, specify `option`. Each `option` is an integer
constant in `orjson`. To specify multiple options, mask them together, e.g.,
`option=orjson.OPT_STRICT_INTEGER | orjson.OPT_NAIVE_UTC`.

##### OPT_NAIVE_UTC

Serialize `datetime.datetime` objects without a `tzinfo` as UTC. This
has no effect on `datetime.datetime` objects that have `tzinfo` set.

```python
>>> import orjson, datetime
>>> orjson.dumps(
        datetime.datetime(1970, 1, 1, 0, 0, 0),
    )
b'"1970-01-01T00:00:00"'
>>> orjson.dumps(
        datetime.datetime(1970, 1, 1, 0, 0, 0),
        option=orjson.OPT_NAIVE_UTC,
    )
b'"1970-01-01T00:00:00+00:00"'
```

##### OPT_OMIT_MICROSECONDS

Do not serialize the `microsecond` field on `datetime.datetime` and
`datetime.time` instances.

```python
>>> import orjson, datetime
>>> orjson.dumps(
        datetime.datetime(1970, 1, 1, 0, 0, 0, 1),
    )
b'"1970-01-01T00:00:00.000001"'
>>> orjson.dumps(
        datetime.datetime(1970, 1, 1, 0, 0, 0, 1),
        option=orjson.OPT_OMIT_MICROSECONDS,
    )
b'"1970-01-01T00:00:00"'
```

##### OPT_SERIALIZE_DATACLASS

Serialize `dataclasses.dataclass` instances. For more, see
[dataclass](https://github.com/ijl/orjson#dataclass).

##### OPT_SERIALIZE_UUID

Serialize `uuid.UUID` instances. For more, see
[uuid](https://github.com/ijl/orjson#UUID).

##### OPT_STRICT_INTEGER

Enforce 53-bit limit on integers. The limit is otherwise 64 bits, the same as
the Python standard library. For more, see [int](https://github.com/ijl/orjson#int).

##### OPT_UTC_Z

Serialize a UTC timezone on `datetime.datetime` instances as `Z` instead
of `+00:00`.

```python
>>> import orjson, datetime
>>> orjson.dumps(
        datetime.datetime(1970, 1, 1, 0, 0, 0, tzinfo=datetime.timezone.utc),
    )
b'"1970-01-01T00:00:00+00:00"'
>>> orjson.dumps(
        datetime.datetime(1970, 1, 1, 0, 0, 0, tzinfo=datetime.timezone.utc),
        option=orjson.OPT_UTC_Z
    )
b'"1970-01-01T00:00:00Z"'
```

### Deserialize

```python
def loads(__obj: Union[bytes, bytearray, str]) -> Any: ...
```

`loads()` deserializes JSON to Python objects. It deserializes to `dict`,
`list`, `int`, `float`, `str`, `bool`, and `None` objects.

`bytes`, `bytearray`, and `str` input are accepted. If the input exists as
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

## Types

### dataclass

orjson serializes instances of `dataclasses.dataclass` natively. It serializes
instances 30x as fast as other libraries and avoids a severe slowdown seen
in other libraries compared to serializing `dict`. To serialize
instances, specify `option=orjson.OPT_SERIALIZE_DATACLASS`. The option
is required so that users may continue to use `default` until the
implementation allows customizing instances' serialization.

It is supported to pass all variants of dataclasses, including dataclasses
using `__slots__` (which yields a modest performance improvement), frozen
dataclasses, those with optional or default attributes, and subclasses.

| Library    | dict (ms)   | dataclass (ms)   | dataclass vs. dict   | vs. orjson   |
|------------|-------------|------------------|----------------------|--------------|
| orjson     | 0.10        | 0.19             | -46%                 | 1            |
| ujson      |             |                  |                      |              |
| rapidjson  | 0.24        | 6.48             | -96%                 | 33           |
| simplejson | 1.06        | 7.94             | -86%                 | 40           |
| json       | 0.92        | 7.32             | -87%                 | 37           |

This measures orjson serializing instances natively and other libraries using
`default` to serialize the output of `dataclasses.asdict()`. This can be
reproduced using the `pydataclass` script.

Dataclasses are serialized as maps, with every attribute serialized and in
the order given on class definition:

```python
>>> import dataclasses, orjson, typing

@dataclasses.dataclass
class Member:
    id: int
    active: bool = dataclasses.field(default=False)

@dataclasses.dataclass
class Object:
    id: int
    name: str
    members: typing.List[Member]

>>> orjson.dumps(
        Object(1, "a", [Member(1, True), Member(2)]),
        option=orjson.OPT_SERIALIZE_DATACLASS,
    )
b'{"id":1,"name":"a","members":[{"id":1,"active":true},{"id":2,"active":false}]}'
```
Users may wish to control how dataclass instances are serialized, e.g.,
to not serialize an attribute or to change the name of an
attribute when serialized. orjson may implement support using the
metadata mapping on `field` attributes,
e.g., `field(metadata={"json_serialize": False})`, if use cases are clear.

### datetime

orjson serializes `datetime.datetime` objects to
[RFC 3339](https://tools.ietf.org/html/rfc3339) format,
e.g., "1970-01-01T00:00:00+00:00". This is a subset of ISO 8601 and
compatible with `isoformat()` in the standard library.

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

`datetime.datetime` supports instances with a `tzinfo` that is `None`,
`datetime.timezone.utc` or a timezone instance from
the `pendulum`, `pytz`, or `dateutil`/`arrow` libraries.

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

### float

orjson serializes and deserializes floats with no loss of precision and
consistent rounding. The same behavior is observed in rapidjson, simplejson,
and json. ujson is inaccurate in both serialization and deserialization,
i.e., it modifies the data.

`orjson.dumps()` serializes Nan, Infinity, and -Infinity, which are not
compliant JSON, as `null`:

```python
>>> import orjson, ujson, rapidjson, json
>>> orjson.dumps([float("NaN"), float("Infinity"), float("-Infinity")])
b'[null,null,null]'
>>> ujson.dumps([float("NaN"), float("Infinity"), float("-Infinity")])
OverflowError: Invalid Inf value when encoding double
>>> rapidjson.dumps([float("NaN"), float("Infinity"), float("-Infinity")])
'[NaN,Infinity,-Infinity]'
>>> json.dumps([float("NaN"), float("Infinity"), float("-Infinity")])
'[NaN, Infinity, -Infinity]'
```

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

### str

orjson is strict about UTF-8 conformance. This is stricter than the standard
library's json module, which will serialize and deserialize UTF-16 surrogates,
e.g., "\ud800", that are invalid UTF-8.

If `orjson.dumps()` is given a `str` that does not contain valid UTF-8,
`orjson.JSONEncodeError` is raised. If `loads()` receives invalid UTF-8,
`orjson.JSONDecodeError` is raised.

orjson and rapidjson are the only compared JSON libraries to consistently
error on bad input.

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
>>> orjson.loads('"\\ud800"')
JSONDecodeError: unexpected end of hex escape at line 1 column 8: line 1 column 1 (char 0)
>>> ujson.loads('"\\ud800"')
''
>>> rapidjson.loads('"\\ud800"')
ValueError: Parse error at offset 1: The surrogate pair in string is invalid.
>>> json.loads('"\\ud800"')
'\ud800'
```

### UUID

orjson serializes `uuid.UUID` instances to
[RFC 4122](https://tools.ietf.org/html/rfc4122) format, e.g.,
"f81d4fae-7dec-11d0-a765-00a0c91e6bf6". This requires specifying
`option=orjson.OPT_SERIALIZE_UUID`.

``` python
>>> import orjson, uuid
>>> orjson.dumps(
    uuid.UUID('f81d4fae-7dec-11d0-a765-00a0c91e6bf6'),
    option=orjson.OPT_SERIALIZE_UUID,
)
b'"f81d4fae-7dec-11d0-a765-00a0c91e6bf6"'
>>> orjson.dumps(
    uuid.uuid5(uuid.NAMESPACE_DNS, "python.org"),
    option=orjson.OPT_SERIALIZE_UUID,
)
b'"886313e1-3b8a-5372-9b90-0c9aee199e5d"'
```

## Testing

The library has comprehensive tests. There are tests against fixtures in the
[JSONTestSuite](https://github.com/nst/JSONTestSuite) and
[nativejson-benchmark](https://github.com/miloyip/nativejson-benchmark)
repositories. It is tested to not crash against the
[Big List of Naughty Strings](https://github.com/minimaxir/big-list-of-naughty-strings).
It is tested to not leak memory. It is tested to not crash
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
| orjson     |                            0.75 |                  1297.5 |                 1    |
| ujson      |                            2.06 |                   483.5 |                 2.74 |
| rapidjson  |                            2.12 |                   470.7 |                 2.82 |
| simplejson |                            3.55 |                   275.2 |                 4.73 |
| json       |                            3.57 |                   277.8 |                 4.75 |

#### twitter.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            3.29 |                   302.3 |                 1    |
| ujson      |                            3.65 |                   281.2 |                 1.11 |
| rapidjson  |                            5.6  |                   179.1 |                 1.7  |
| simplejson |                            5.19 |                   188.3 |                 1.58 |
| json       |                            5.62 |                   184.2 |                 1.71 |

#### github.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.08 |                 12363.5 |                 1    |
| ujson      |                            0.2  |                  4834.3 |                 2.55 |
| rapidjson  |                            0.23 |                  4385.4 |                 2.84 |
| simplejson |                            0.42 |                  2360.3 |                 5.28 |
| json       |                            0.36 |                  2709.1 |                 4.53 |

#### github.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.25 |                  3992.4 |                 1    |
| ujson      |                            0.32 |                  3065.1 |                 1.28 |
| rapidjson  |                            0.42 |                  2400.2 |                 1.68 |
| simplejson |                            0.3  |                  3293.5 |                 1.21 |
| json       |                            0.38 |                  2410   |                 1.54 |

#### citm_catalog.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            1.27 |                   746.2 |                 1    |
| ujson      |                            3.63 |                   257.1 |                 2.86 |
| rapidjson  |                            3.52 |                   279.8 |                 2.77 |
| simplejson |                           14.37 |                    66.6 |                11.31 |
| json       |                            8.28 |                   120.2 |                 6.52 |

#### citm_catalog.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            5.61 |                   175.8 |                 1    |
| ujson      |                            6.78 |                   146.8 |                 1.21 |
| rapidjson  |                            7.71 |                   129.4 |                 1.37 |
| simplejson |                            9.01 |                   108.8 |                 1.61 |
| json       |                            8.49 |                   116.1 |                 1.51 |

#### canada.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            5.28 |                   189.6 |                 1    |
| ujson      |                                 |                         |                      |
| rapidjson  |                           69.38 |                    14.3 |                13.14 |
| simplejson |                           99.43 |                     9.4 |                18.84 |
| json       |                           76.44 |                    12.9 |                14.48 |

#### canada.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                           22.22 |                    45.1 |                 1    |
| ujson      |                                 |                         |                      |
| rapidjson  |                           44.56 |                    21.4 |                 2.01 |
| simplejson |                           42.99 |                    23.2 |                 1.93 |
| json       |                           44.69 |                    21.4 |                 2.01 |

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
| orjson     |                       12.9 |                             2.8 |
| ujson      |                       12.8 |                             4.6 |
| rapidjson  |                       14.5 |                             6.5 |
| simplejson |                       13.1 |                             2.7 |
| json       |                       12.5 |                             2.4 |

#### github.json

| Library    |   import, read() RSS (MiB) |   loads() increase in RSS (MiB) |
|------------|----------------------------|---------------------------------|
| orjson     |                       12.3 |                             0.3 |
| ujson      |                       12.6 |                             0.5 |
| rapidjson  |                       13.9 |                             0.4 |
| simplejson |                       12.5 |                             0.3 |
| json       |                       11.7 |                             0.3 |

#### citm_catalog.json

| Library    |   import, read() RSS (MiB) |   loads() increase in RSS (MiB) |
|------------|----------------------------|---------------------------------|
| orjson     |                       13.7 |                             8.5 |
| ujson      |                       13.9 |                            12   |
| rapidjson  |                       15.4 |                            30.2 |
| simplejson |                       14.1 |                            25   |
| json       |                       13.5 |                            24.9 |

#### canada.json

| Library    | import, read() RSS (MiB)   | loads() increase in RSS (MiB)   |
|------------|----------------------------|---------------------------------|
| orjson     | 16.5                       | 17.5                            |
| ujson      |                            |                                 |
| rapidjson  | 17.9                       | 19.6                            |
| simplejson | 16.6                       | 21.3                            |
| json       | 16.0                       | 21.3                            |

### Reproducing

The above was measured using Python 3.7.4 on Linux with orjson 2.1.0,
ujson 1.35, python-rapidson 0.8.0, and simplejson 3.16.0.

The latency results can be reproduced using the `pybench` and `graph`
scripts. The memory results can be reproduced using the `pymem` script.

## License

orjson was written by ijl <<ijl@mailbox.org>>, copyright 2018 - 2020, licensed
under either the Apache 2 or MIT licenses.
