# Changelog

## 2.0.2 - 2019-03-12

### Changed

- Support Python 3.5.

## 2.0.1 - 2019-02-05

### Changed

- Publish Windows wheel.

## 2.0.0 - 2019-01-28

### Added

- `orjson.dumps()` accepts a `default` callable to serialize arbitrary
types.
- `orjson.dumps()` accepts `datetime.datetime`, `datetime.date`,
and `datetime.time`. Each is serialized to an RFC 3339 string.
- `orjson.dumps(..., option=orjson.OPT_NAIVE_UTC)` allows serializing
`datetime.datetime` objects that do not have a timezone set as UTC.
- `orjson.dumps(..., option=orjson.OPT_STRICT_INTEGER)` available to
raise an error on integer values outside the 53-bit range of all JSON
implementations.

### Changed

- `orjson.dumps()` no longer accepts `bytes`.

## 1.3.1 - 2019-01-03

### Fixed

- Handle invalid UTF-8 in str.

## 1.3.0 - 2019-01-02

### Changed

- Performance improvements of 15-25% on serialization, 10% on deserialization.

## 1.2.1 - 2018-12-31

### Fixed

- Fix memory leak in deserializing dict.

## 1.2.0 - 2018-12-16

### Changed

- Performance improvements.

## 1.1.0 - 2018-12-04

### Changed

- Performance improvements.

### Fixed

- Dict key can only be str.

## 1.0.1 - 2018-11-26

### Fixed

- pyo3 bugfix update.

## 1.0.0 - 2018-11-23

### Added

- `orjson.dumps()` function.
- `orjson.loads()` function.
