# orjson

orjson is a fast JSON library for Python.

It supports Python 3.6 and Python 3.7.

## Performance

Serialization performance of orjson is better than ujson, rapidjson, or
json. Deserialization performance is worse to about the same as `ujson`.

This is measured using data from the
[nativejson-benchmark](https://github.com/miloyip/nativejson-benchmark)
repository on Python 3.7.1.

![alt text](doc/twitter-serialize.png "twitter.json serialization")
![alt text](doc/citm_catalog-serialize.png "citm_catalog.json serialization")
![alt text](doc/canada-serialize.png "canada.json serialization")

The above can be reproduced using the `pybench` and `graph` scripts.

## License

orjson is dual licensed under the Apache 2.0 and MIT licenses. It contains
code from the hyperjson and ultrajson libraries. It is implemented using
the [serde_json](https://github.com/serde-rs/json) and
[pyo3](https://github.com/PyO3/pyo3) libraries.
