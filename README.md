# e-json

`e-json` is a high-performance JSON library for Python, forked from [`orjson`](https://github.com/ijl/orjson). It provides fast serialization and deserialization, with additional features for handling big integers and special floating-point values.

## Features

- **Big Integer Support**  
    Parse large integers without loss of precision using the `OPT_BIG_INTEGER` option.

    ```python
    import orjson

    result = orjson.loads(b'100000000000000000001', option=orjson.OPT_BIG_INTEGER)
    print(result)  # 100000000000000000001
    ```

- **NaN as Null**  
    Convert `NaN` values to `null` during deserialization with the `OPT_NAN_AS_NULL` option.

    ```python
    import orjson

    result = orjson.loads('{"x": nan}', option=orjson.OPT_NAN_AS_NULL)
    print(result)  # {'x': None}
    ```

## Installation

```bash
pip install e-json
```

## Usage

`e-json` is a drop-in replacement for `orjson`. Simply import and use the additional options as needed.

## License

See [LICENSE](./LICENSE) for details.
