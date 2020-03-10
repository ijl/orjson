# orjson

orjson is a fast, correct JSON library for Python. It
[benchmarks](https://github.com/ijl/orjson#performance) as the fastest Python
library for JSON and is more correct than the standard json library or other
third-party libraries. It serializes
[dataclass](https://github.com/ijl/orjson#dataclass),
[datetime](https://github.com/ijl/orjson#datetime),
[numpy](https://github.com/ijl/orjson#numpy), and
[UUID](https://github.com/ijl/orjson#UUID) instances natively.

Its features and drawbacks compared to other Python JSON libraries:

* serializes `dataclass` instances 40-50x as fast as other libraries
* serializes `datetime`, `date`, and `time` instances to RFC 3339 format,
e.g., "1970-01-01T00:00:00+00:00"
* serializes `numpy.ndarray` instances 3-10x as fast as other libraries
* pretty prints 10x to 20x as fast as the standard library
* serializes to `bytes` rather than `str`, i.e., is not a drop-in replacement
* serializes `str` without escaping unicode to ASCII, e.g., "好" rather than
"\\\u597d"
* serializes `float` 10x as fast and deserializes twice as fast as other
libraries
* serializes arbitrary types using a `default` hook
* has strict UTF-8 conformance, more correct than the standard library
* has strict JSON conformance in not supporting Nan/Infinity/-Infinity
* has an option for strict JSON conformance on 53-bit integers with default
support for 64-bit
* does not support subclasses by default, requiring use of `default` hook
* does not provide `load()` or `dump()` functions for reading from/writing to
file-like objects

orjson supports CPython 3.6, 3.7, 3.8, and 3.9. It distributes x86_64/amd64
and aarch64/armv8 wheels for Linux. It distributes x86_64/amd64 wheels for
macOS and Windows. orjson does not support PyPy.

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
    5. [numpy](https://github.com/ijl/orjson#numpy)
    6. [str](https://github.com/ijl/orjson#str)
    7. [UUID](https://github.com/ijl/orjson#UUID)
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
`datetime.date`, `datetime.time`, `uuid.UUID`, and `None` instances. It
supports arbitrary types through `default`. It does not serialize subclasses of
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
lambda, or callable class instance. To specify that a type was not
handled by `default`, raise an exception such as `TypeError`.

```python
>>> import orjson, decimal
>>>
def default(obj):
    if isinstance(obj, decimal.Decimal):
        return str(obj)
    raise TypeError

>>> orjson.dumps(decimal.Decimal("0.0842389659712649442845"))
JSONEncodeError: Type is not JSON serializable: decimal.Decimal
>>> orjson.dumps(decimal.Decimal("0.0842389659712649442845"), default=default)
b'"0.0842389659712649442845"'
>>> orjson.dumps({1, 2}, default=default)
orjson.JSONEncodeError: Type is not JSON serializable: set
```

The `default` callable may return an object that itself
must be handled by `default` up to 254 times before an exception
is raised.

It is important that `default` raise an exception if a type cannot be handled.
Python otherwise implicitly returns `None`, which appears to the caller
like a legitimate value and is serialized:

```python
>>> import orjson, json, rapidjson
>>>
def default(obj):
    if isinstance(obj, decimal.Decimal):
        return str(obj)

>>> orjson.dumps({"set":{1, 2}}, default=default)
b'{"set":null}'
>>> json.dumps({"set":{1, 2}}, default=default)
'{"set":null}'
>>> rapidjson.dumps({"set":{1, 2}}, default=default)
'{"set":null}'
```

#### option

To modify how data is serialized, specify `option`. Each `option` is an integer
constant in `orjson`. To specify multiple options, mask them together, e.g.,
`option=orjson.OPT_STRICT_INTEGER | orjson.OPT_NAIVE_UTC`.

##### OPT_INDENT_2

Pretty-print output with an indent of two spaces. This is equivalent to
`indent=2` in the standard library. Pretty printing is slower and the output
larger. orjson is the fastest compared library at pretty printing and has
much less of a slowdown to pretty print than the standard library does. This
option is compatible with all other options.

```python
>>> import orjson
>>> orjson.dumps({"a": "b", "c": {"d": True}, "e": [1, 2]})
b'{"a":"b","c":{"d":true},"e":[1,2]}'
>>> orjson.dumps(
    {"a": "b", "c": {"d": True}, "e": [1, 2]},
    option=orjson.OPT_INDENT_2
)
b'{\n  "a": "b",\n  "c": {\n    "d": true\n  },\n  "e": [\n    1,\n    2\n  ]\n}'
```

If displayed, the indentation and linebreaks appear like this:

```json
{
  "a": "b",
  "c": {
    "d": true
  },
  "e": [
    1,
    2
  ]
}
```

This measures serializing the github.json fixture as compact (52KiB) or
pretty (64KiB):

| Library    |   compact (ms) | pretty (ms)   | vs. orjson   |
|------------|----------------|---------------|--------------|
| orjson     |           0.07 | 0.09          | 1.0          |
| ujson      |           0.17 | 0.18          | 2.1          |
| rapidjson  |           0.25 |               |              |
| simplejson |           0.42 | 1.50          | 17.0         |
| json       |           0.41 | 1.29          | 14.6         |

This measures serializing the citm_catalog.json fixture, more of a worst
case due to the amount of nesting and newlines, as compact (489KiB) or
pretty (1.1MiB):

| Library    |   compact (ms) | pretty (ms)   | vs. orjson   |
|------------|----------------|---------------|--------------|
| orjson     |           1.11 | 2.49          | 1.0          |
| ujson      |           3.44 | 5.19          | 2.1          |
| rapidjson  |           3.61 |               |              |
| simplejson |          11.43 | 77.88         | 31.3         |
| json       |           7.41 | 56.34         | 22.6         |

rapidjson is blank because it does not support pretty printing. This can be
reproduced using the `pyindent` script.

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

##### OPT_NON_STR_KEYS

Serialize `dict` keys of type other than `str`. This allows `dict` keys
to be one of `str`, `int`, `float`, `bool`, `None`, `datetime.datetime`,
`datetime.date`, `datetime.time`, and `uuid.UUID`. For comparison,
the standard library serializes `str`, `int`, `float`, `bool` or `None` by
default. orjson benchmarks as being faster at serializing non-`str` keys
than other libraries. This option is slower for `str` keys than the default
and is not recommended generally.

```python
>>> import orjson, datetime, uuid
>>> orjson.dumps(
        {uuid.UUID("7202d115-7ff3-4c81-a7c1-2a1f067b1ece"): [1, 2, 3]},
        option=orjson.OPT_NON_STR_KEYS,
    )
b'{"7202d115-7ff3-4c81-a7c1-2a1f067b1ece":[1,2,3]}'
>>> orjson.dumps(
        {datetime.datetime(1970, 1, 1, 0, 0, 0): [1, 2, 3]},
        option=orjson.OPT_NON_STR_KEYS | orjson.OPT_NAIVE_UTC,
    )
b'{"1970-01-01T00:00:00+00:00":[1,2,3]}'
```

These types are generally serialized how they would be as
values, e.g., `datetime.datetime` is still an RFC 3339 string and respects
options affecting it. The exception is that `int` serialization does not
respect `OPT_STRICT_INTEGER`.

This option has the risk of creating duplicate keys. This is because non-`str`
objects may serialize to the same `str` as an existing key, e.g.,
`{"1": true, 1: false}`. The last key to be inserted to the `dict` will be
serialized last and a JSON deserializer will presumably take the last
occurrence of a key (in the above, `false`). The first value will be lost.

This option is compatible with `orjson.OPT_SORT_KEYS`. If sorting is used,
note the sort is unstable and will be unpredictable for duplicate keys.

```python
>>> import orjson, datetime
>>> orjson.dumps(
    {"other": 1, datetime.date(1970, 1, 5): 2, datetime.date(1970, 1, 3): 3},
    option=orjson.OPT_NON_STR_KEYS | orjson.OPT_SORT_KEYS
)
b'{"1970-01-03":3,"1970-01-05":2,"other":1}'
```

This measures serializing 589KiB of JSON comprising a `list` of 100 `dict`
in which each `dict` has both 365 randomly-sorted `int` keys representing epoch
timestamps as well as one `str` key and the value for each key is a
single integer. In "str keys", the keys were converted to `str` before
serialization, and orjson still specifes `option=orjson.OPT_NON_STR_KEYS`
(which is always somewhat slower).

| Library    |   str keys (ms) | int keys (ms)   | int keys sorted (ms)   |
|------------|-----------------|-----------------|------------------------|
| orjson     |            2.32 | 2.85            | 5.80                   |
| ujson      |            3.15 | 5.07            |                        |
| rapidjson  |            4.73 |                 |                        |
| simplejson |           11.74 | 15.58           | 24.38                  |
| json       |            6.94 | 8.97            |                        |

ujson is blank for sorting because it segfaults. json is blank because it
raises `TypeError` on attempting to sort before converting all keys to `str`.
rapidjson is blank because it does not support non-`str` keys. This can
be reproduced using the `pynonstr` script.

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

##### OPT_SERIALIZE_NUMPY

Serialize `numpy.ndarray` instances. For more, see
[numpy](https://github.com/ijl/orjson#numpy).

##### OPT_SERIALIZE_UUID

Serialize `uuid.UUID` instances. For more, see
[UUID](https://github.com/ijl/orjson#UUID).

##### OPT_SORT_KEYS

Serialize `dict` keys in sorted order. The default is to serialize in an
unspecified order. This is equivalent to `sort_keys=True` in the standard
library.

This can be used to ensure the order is deterministic for hashing or tests.
It has a substantial performance penalty and is not recommended in general.

```python
>>> import orjson
>>> orjson.dumps({"b": 1, "c": 2, "a": 3})
b'{"b":1,"c":2,"a":3}'
>>> orjson.dumps({"b": 1, "c": 2, "a": 3}, option=orjson.OPT_SORT_KEYS)
b'{"a":3,"b":1,"c":2}'
```

This measures serializing the twitter.json fixture unsorted and sorted:

| Library    |   unsorted (ms) |   sorted (ms) |   vs. orjson |
|------------|-----------------|---------------|--------------|
| orjson     |            0.96 |          1.49 |            1 |
| ujson      |            2.57 |          3.62 |            2 |
| rapidjson  |            3.43 |          4.44 |            2 |
| simplejson |            4.83 |          6.97 |            4 |
| json       |            4.77 |          6.29 |            4 |

The benchmark can be reproduced using the `pysort` script.

The sorting is not collation/locale-aware:

```python
>>> import orjson
>>> orjson.dumps({"a": 1, "ä": 2, "A": 3}, option=orjson.OPT_SORT_KEYS)
b'{"A":3,"a":1,"\xc3\xa4":2}'
```

This is the same sorting behavior as the standard library, rapidjson,
simplejson, and ujson.

`dataclass` also serialize as maps but this has no effect on them.

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
instances 40-50x as fast as other libraries and avoids a severe slowdown seen
in other libraries compared to serializing `dict`. To serialize
instances, specify `option=orjson.OPT_SERIALIZE_DATACLASS`. The option
is required so that users may continue to use `default` until the
implementation allows customizing instances' serialization.

It is supported to pass all variants of dataclasses, including dataclasses
using `__slots__`, frozen dataclasses, those with optional or default
attributes, and subclasses. There is a performance benefit to not
using `__slots__`.

| Library    | dict (ms)   | dataclass (ms)   | vs. orjson   |
|------------|-------------|------------------|--------------|
| orjson     | 1.64        | 1.86             | 1            |
| ujson      |             |                  |              |
| rapidjson  | 3.90        | 86.75            | 46           |
| simplejson | 17.40       | 103.84           | 55           |
| json       | 12.90       | 98.37            | 52           |

This measures serializing 555KiB of JSON, orjson natively and other libraries
using `default` to serialize the output of `dataclasses.asdict()`. This can be
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

orjson serializes and deserializes double precision floats with no loss of
precision and consistent rounding. The same behavior is observed in rapidjson,
simplejson, and json. ujson is inaccurate in both serialization and
deserialization, i.e., it modifies the data.

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

### numpy

orjson natively serializes `numpy.ndarray` instances. Arrays may have a
`dtype` of `numpy.bool`, `numpy.float32`, `numpy.float64`, `numpy.int32`,
`numpy.int64`, `numpy.uint32`, `numpy.uint64`, `numpy.uintp`, or `numpy.intp`.
orjson is faster than all compared libraries at serializing
numpy instances. Serializing numpy data requires specifying
`option=orjson.OPT_SERIALIZE_NUMPY`.

```python
>>> import orjson, numpy
>>> orjson.dumps(
        numpy.array([[1, 2, 3], [4, 5, 6]]),
        option=orjson.OPT_SERIALIZE_NUMPY,
)
b'[[1,2,3],[4,5,6]]'
```

The array must be a contiguous C array (`C_CONTIGUOUS`) and one of the
supported datatypes. Individual items (e.g., `numpy.float64(1)`) are
not supported.

If an array is not a contiguous C array or contains an supported datatype,
orjson falls through to `default`. In `default`, `obj.tolist()` can be
specified. If an array is malformed, which is not expected,
`orjson.JSONEncodeError` is raised.

This measures serializing 92MiB of JSON from an `numpy.ndarray` with
dimensions of `(50000, 100)` and `numpy.float64` values:

| Library    | Latency (ms)   | RSS diff (MiB)   | vs. orjson   |
|------------|----------------|------------------|--------------|
| orjson     | 286            | 182              | 1            |
| nujson     |                |                  |              |
| rapidjson  | 3,582          | 270              | 12           |
| simplejson | 3,494          | 259              | 12           |
| json       | 3,476          | 260              | 12           |

This measures serializing 100MiB of JSON from an `numpy.ndarray` with
dimensions of `(100000, 100)` and `numpy.int32` values:

| Library    | Latency (ms)   |   RSS diff (MiB) |   vs. orjson |
|------------|----------------|------------------|--------------|
| orjson     | 225            |              198 |            1 |
| nujson     | 2,240          |              246 |            9 |
| rapidjson  | 2,235          |              462 |            9 |
| simplejson | 1,686          |              430 |            7 |
| json       | 1,626          |              430 |            7 |

This measures serializing 53MiB of JSON from an `numpy.ndarray` with
dimensions of `(100000, 100)` and `numpy.bool` values:

| Library    | Latency (ms)   |   RSS diff (MiB) |   vs. orjson |
|------------|----------------|------------------|--------------|
| orjson     | 121            |               53 |            1 |
| nujson     | 5,958          |               43 |           49 |
| rapidjson  | 482            |              101 |            3 |
| simplejson | 671            |              126 |            5 |
| json       | 609            |              127 |            5 |

In these benchmarks, nujson is used instead of ujson, orjson and nujson
serialize natively, and the other libraries use `ndarray.tolist()`. `nujson`
is blank when it did not roundtrip the data accurately. The RSS
column measures peak memory usage during serialization. The odd
bool result for nujson is consistent.

orjson does not have an installation or compilation dependency on numpy. The
implementation is independent, reading `numpy.ndarray` using
`PyArrayInterface`.

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

To make a best effort at deserializing bad input, first decode `bytes` using
the `replace` or `lossy` argument for `errors`:

```python
>>> import orjson
>>> orjson.loads(b'"\xed\xa0\x80"')
JSONDecodeError: str is not valid UTF-8: surrogates not allowed
>>> orjson.loads(b'"\xed\xa0\x80"'.decode("utf-8", "replace"))
'���'
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
| orjson     |                            0.74 |                  1358.5 |                 1    |
| ujson      |                            1.95 |                   511.1 |                 2.65 |
| rapidjson  |                            2.58 |                   387.1 |                 3.51 |
| simplejson |                            3.49 |                   287   |                 4.74 |
| json       |                            3.4  |                   294.4 |                 4.61 |

#### twitter.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            2.74 |                   364.5 |                 1    |
| ujson      |                            3.01 |                   332.7 |                 1.1  |
| rapidjson  |                            3.98 |                   251.1 |                 1.45 |
| simplejson |                            3.64 |                   275.5 |                 1.33 |
| json       |                            4.27 |                   234.5 |                 1.56 |

#### github.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.08 |                 12278.6 |                 1    |
| ujson      |                            0.19 |                  5243.6 |                 2.33 |
| rapidjson  |                            0.29 |                  3427.9 |                 3.57 |
| simplejson |                            0.47 |                  2125.3 |                 5.77 |
| json       |                            0.36 |                  2774.1 |                 4.4  |

#### github.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            0.23 |                  4300.7 |                 1    |
| ujson      |                            0.29 |                  3459.3 |                 1.24 |
| rapidjson  |                            0.33 |                  2980.8 |                 1.43 |
| simplejson |                            0.31 |                  3186.4 |                 1.36 |
| json       |                            0.35 |                  2892.5 |                 1.5  |

#### citm_catalog.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            1.21 |                   835   |                 1    |
| ujson      |                            3.33 |                   299.9 |                 2.76 |
| rapidjson  |                            3.8  |                   264.8 |                 3.14 |
| simplejson |                           12.12 |                    82.7 |                10.02 |
| json       |                            7.81 |                   129   |                 6.46 |

#### citm_catalog.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            5.25 |                   190.5 |                 1    |
| ujson      |                            6.49 |                   154.1 |                 1.24 |
| rapidjson  |                            8    |                   124.9 |                 1.52 |
| simplejson |                            7.94 |                   125.7 |                 1.51 |
| json       |                            8.62 |                   116.1 |                 1.64 |

#### canada.json serialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                            5.54 |                   180.6 |                 1    |
| ujson      |                                 |                         |                      |
| rapidjson  |                           70.29 |                    14.4 |                12.69 |
| simplejson |                           90.03 |                    11.2 |                16.25 |
| json       |                           73.39 |                    13.6 |                13.25 |

#### canada.json deserialization

| Library    |   Median latency (milliseconds) |   Operations per second |   Relative (latency) |
|------------|---------------------------------|-------------------------|----------------------|
| orjson     |                           19.6  |                    51   |                 1    |
| ujson      |                                 |                         |                      |
| rapidjson  |                           42.02 |                    23.9 |                 2.14 |
| simplejson |                           40.19 |                    24.9 |                 2.05 |
| json       |                           41.5  |                    24.1 |                 2.12 |

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
| orjson     |                       13.7 |                             2.4 |
| ujson      |                       13.4 |                             4   |
| rapidjson  |                       14.8 |                             6.5 |
| simplejson |                       13.3 |                             2.5 |
| json       |                       12.8 |                             2.6 |

#### github.json

| Library    |   import, read() RSS (MiB) |   loads() increase in RSS (MiB) |
|------------|----------------------------|---------------------------------|
| orjson     |                       12.9 |                             0.3 |
| ujson      |                       12.5 |                             0.4 |
| rapidjson  |                       13.9 |                             0.6 |
| simplejson |                       12.5 |                             0.3 |
| json       |                       12.1 |                             0.4 |

#### citm_catalog.json

| Library    |   import, read() RSS (MiB) |   loads() increase in RSS (MiB) |
|------------|----------------------------|---------------------------------|
| orjson     |                       14.6 |                             7.7 |
| ujson      |                       14.5 |                            10.8 |
| rapidjson  |                       15.7 |                            26.1 |
| simplejson |                       14.3 |                            16   |
| json       |                       14.1 |                            24.1 |

#### canada.json

| Library    | import, read() RSS (MiB)   | loads() increase in RSS (MiB)   |
|------------|----------------------------|---------------------------------|
| orjson     | 17.1                       | 15.7                            |
| ujson      |                            |                                 |
| rapidjson  | 18.1                       | 17.9                            |
| simplejson | 16.8                       | 19.6                            |
| json       | 16.5                       | 19.5                            |

### Reproducing

The above was measured using Python 3.8.1 on Linux with orjson 2.2.1,
ujson 1.35, python-rapidson 0.9.1, and simplejson 3.17.0.

The latency results can be reproduced using the `pybench` and `graph`
scripts. The memory results can be reproduced using the `pymem` script.

## License

orjson was written by ijl <<ijl@mailbox.org>>, copyright 2018 - 2020, licensed
under either the Apache 2 or MIT licenses.
