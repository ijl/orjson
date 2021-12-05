# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import pytest

import orjson

try:
    import numpy
except ImportError:
    numpy = None  # type: ignore


def numpy_default(obj):
    return obj.tolist()


@pytest.mark.skipif(numpy is None, reason="numpy is not installed")
class NumpyTests(unittest.TestCase):
    def test_numpy_array_d1_uintp(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([0, 18446744073709551615], numpy.uintp),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[0,18446744073709551615]",
        )

    def test_numpy_array_d1_intp(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([-9223372036854775807, 9223372036854775807], numpy.intp),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[-9223372036854775807,9223372036854775807]",
        )

    def test_numpy_array_d1_i64(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([-9223372036854775807, 9223372036854775807], numpy.int64),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[-9223372036854775807,9223372036854775807]",
        )

    def test_numpy_array_d1_u64(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([0, 18446744073709551615], numpy.uint64),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[0,18446744073709551615]",
        )

    def test_numpy_array_d1_i8(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([-128, 127], numpy.int8),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[-128,127]",
        )

    def test_numpy_array_d1_u8(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([0, 255], numpy.uint8),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[0,255]",
        )

    def test_numpy_array_d1_i32(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([-2147483647, 2147483647], numpy.int32),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[-2147483647,2147483647]",
        )

    def test_numpy_array_d1_u32(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([0, 4294967295], numpy.uint32),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[0,4294967295]",
        )

    def test_numpy_array_d1_f32(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([1.0, 3.4028235e38], numpy.float32),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[1.0,3.4028235e38]",
        )

    def test_numpy_array_d1_f64(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([1.0, 1.7976931348623157e308], numpy.float64),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[1.0,1.7976931348623157e308]",
        )

    def test_numpy_array_d1_bool(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([True, False, False, True]),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[true,false,false,true]",
        )

    def test_numpy_array_d1_datetime64_years(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("1"),
                        numpy.datetime64("970"),
                        numpy.datetime64("1920"),
                        numpy.datetime64("1971"),
                        numpy.datetime64("2021"),
                        numpy.datetime64("2022"),
                        numpy.datetime64("2023"),
                        numpy.datetime64("9999"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["0001-01-01T00:00:00","0970-01-01T00:00:00","1920-01-01T00:00:00","1971-01-01T00:00:00","2021-01-01T00:00:00","2022-01-01T00:00:00","2023-01-01T00:00:00","9999-01-01T00:00:00"]',
        )

    def test_numpy_array_d1_datetime64_months(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01"),
                        numpy.datetime64("2022-01"),
                        numpy.datetime64("2023-01"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["2021-01-01T00:00:00","2022-01-01T00:00:00","2023-01-01T00:00:00"]',
        )

    def test_numpy_array_d1_datetime64_days(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01-01"),
                        numpy.datetime64("2021-01-01"),
                        numpy.datetime64("2021-01-01"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["2021-01-01T00:00:00","2021-01-01T00:00:00","2021-01-01T00:00:00"]',
        )

    def test_numpy_array_d1_datetime64_hours(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01-01T00"),
                        numpy.datetime64("2021-01-01T01"),
                        numpy.datetime64("2021-01-01T02"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["2021-01-01T00:00:00","2021-01-01T01:00:00","2021-01-01T02:00:00"]',
        )

    def test_numpy_array_d1_datetime64_minutes(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01-01T00:00"),
                        numpy.datetime64("2021-01-01T00:01"),
                        numpy.datetime64("2021-01-01T00:02"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["2021-01-01T00:00:00","2021-01-01T00:01:00","2021-01-01T00:02:00"]',
        )

    def test_numpy_array_d1_datetime64_seconds(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01-01T00:00:00"),
                        numpy.datetime64("2021-01-01T00:00:01"),
                        numpy.datetime64("2021-01-01T00:00:02"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["2021-01-01T00:00:00","2021-01-01T00:00:01","2021-01-01T00:00:02"]',
        )

    def test_numpy_array_d1_datetime64_milliseconds(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01-01T00:00:00"),
                        numpy.datetime64("2021-01-01T00:00:00.172"),
                        numpy.datetime64("2021-01-01T00:00:00.567"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["2021-01-01T00:00:00","2021-01-01T00:00:00.172000","2021-01-01T00:00:00.567000"]',
        )

    def test_numpy_array_d1_datetime64_microseconds(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01-01T00:00:00"),
                        numpy.datetime64("2021-01-01T00:00:00.172"),
                        numpy.datetime64("2021-01-01T00:00:00.567891"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["2021-01-01T00:00:00","2021-01-01T00:00:00.172000","2021-01-01T00:00:00.567891"]',
        )

    def test_numpy_array_d1_datetime64_nanoseconds(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01-01T00:00:00"),
                        numpy.datetime64("2021-01-01T00:00:00.172"),
                        numpy.datetime64("2021-01-01T00:00:00.567891234"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'["2021-01-01T00:00:00","2021-01-01T00:00:00.172000","2021-01-01T00:00:00.567891"]',
        )

    def test_numpy_array_d1_datetime64_picoseconds(self):
        try:
            orjson.dumps(
                numpy.array(
                    [
                        numpy.datetime64("2021-01-01T00:00:00"),
                        numpy.datetime64("2021-01-01T00:00:00.172"),
                        numpy.datetime64("2021-01-01T00:00:00.567891234567"),
                    ]
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            )
            assert False
        except TypeError as exc:
            self.assertEqual(
                str(exc),
                "unsupported numpy.datetime64 unit: picoseconds",
            )

    def test_numpy_array_d2_i64(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([[1, 2, 3], [4, 5, 6]], numpy.int64),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[[1,2,3],[4,5,6]]",
        )

    def test_numpy_array_d2_f64(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], numpy.float64),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[[1.0,2.0,3.0],[4.0,5.0,6.0]]",
        )

    def test_numpy_array_d3_i8(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.int8),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[[[1,2],[3,4]],[[5,6],[7,8]]]",
        )

    def test_numpy_array_d3_u8(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.uint8),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[[[1,2],[3,4]],[[5,6],[7,8]]]",
        )

    def test_numpy_array_d3_i32(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.int32),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[[[1,2],[3,4]],[[5,6],[7,8]]]",
        )

    def test_numpy_array_d3_i64(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array([[[1, 2], [3, 4], [5, 6], [7, 8]]], numpy.int64),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[[[1,2],[3,4],[5,6],[7,8]]]",
        )

    def test_numpy_array_d3_f64(self):
        self.assertEqual(
            orjson.dumps(
                numpy.array(
                    [[[1.0, 2.0], [3.0, 4.0]], [[5.0, 6.0], [7.0, 8.0]]], numpy.float64
                ),
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b"[[[1.0,2.0],[3.0,4.0]],[[5.0,6.0],[7.0,8.0]]]",
        )

    def test_numpy_array_fortran(self):
        array = numpy.array([[1, 2], [3, 4]], order="F")
        assert array.flags["F_CONTIGUOUS"] == True
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY)
        self.assertEqual(
            orjson.dumps(
                array, default=numpy_default, option=orjson.OPT_SERIALIZE_NUMPY
            ),
            orjson.dumps(array.tolist()),
        )

    def test_numpy_array_non_contiguous_message(self):
        array = numpy.array([[1, 2], [3, 4]], order="F")
        assert array.flags["F_CONTIGUOUS"] == True
        try:
            orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY)
            assert False
        except TypeError as exc:
            self.assertEqual(
                str(exc),
                "numpy array is not C contiguous; use ndarray.tolist() in default",
            )

    def test_numpy_array_unsupported_dtype(self):
        array = numpy.array([[1, 2], [3, 4]], numpy.float16)
        with self.assertRaises(orjson.JSONEncodeError) as cm:
            orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY)
        assert str(cm.exception) == "unsupported datatype in numpy array"
        self.assertEqual(
            orjson.dumps(
                array, default=numpy_default, option=orjson.OPT_SERIALIZE_NUMPY
            ),
            orjson.dumps(array.tolist()),
        )

    def test_numpy_array_d1(self):
        array = numpy.array([1])
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

    def test_numpy_array_d2(self):
        array = numpy.array([[1]])
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

    def test_numpy_array_d3(self):
        array = numpy.array([[[1]]])
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

    def test_numpy_array_d4(self):
        array = numpy.array([[[[1]]]])
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

    def test_numpy_array_4_stride(self):
        array = numpy.random.rand(4, 4, 4, 4)
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

    def test_numpy_array_dimension_zero(self):
        array = numpy.array(0)
        assert array.ndim == 0
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY)

        array = numpy.empty((0, 4, 2))
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

        array = numpy.empty((4, 0, 2))
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

        array = numpy.empty((2, 4, 0))
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

    def test_numpy_array_dimension_max(self):
        array = numpy.random.rand(
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
            1,
        )
        assert array.ndim == 32
        self.assertEqual(
            orjson.loads(
                orjson.dumps(
                    array,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                )
            ),
            array.tolist(),
        )

    def test_numpy_scalar_int8(self):
        self.assertEqual(
            orjson.dumps(numpy.int8(0), option=orjson.OPT_SERIALIZE_NUMPY), b"0"
        )
        self.assertEqual(
            orjson.dumps(numpy.int8(127), option=orjson.OPT_SERIALIZE_NUMPY),
            b"127",
        )
        self.assertEqual(
            orjson.dumps(numpy.int8(--128), option=orjson.OPT_SERIALIZE_NUMPY),
            b"-128",
        )

    def test_numpy_scalar_int32(self):
        self.assertEqual(
            orjson.dumps(numpy.int32(1), option=orjson.OPT_SERIALIZE_NUMPY), b"1"
        )
        self.assertEqual(
            orjson.dumps(numpy.int32(2147483647), option=orjson.OPT_SERIALIZE_NUMPY),
            b"2147483647",
        )
        self.assertEqual(
            orjson.dumps(numpy.int32(-2147483648), option=orjson.OPT_SERIALIZE_NUMPY),
            b"-2147483648",
        )

    def test_numpy_scalar_int64(self):
        self.assertEqual(
            orjson.dumps(
                numpy.int64(-9223372036854775808), option=orjson.OPT_SERIALIZE_NUMPY
            ),
            b"-9223372036854775808",
        )
        self.assertEqual(
            orjson.dumps(
                numpy.int64(9223372036854775807), option=orjson.OPT_SERIALIZE_NUMPY
            ),
            b"9223372036854775807",
        )

    def test_numpy_scalar_uint8(self):
        self.assertEqual(
            orjson.dumps(numpy.uint8(0), option=orjson.OPT_SERIALIZE_NUMPY), b"0"
        )
        self.assertEqual(
            orjson.dumps(numpy.uint8(255), option=orjson.OPT_SERIALIZE_NUMPY),
            b"255",
        )

    def test_numpy_scalar_uint32(self):
        self.assertEqual(
            orjson.dumps(numpy.uint32(0), option=orjson.OPT_SERIALIZE_NUMPY), b"0"
        )
        self.assertEqual(
            orjson.dumps(numpy.uint32(4294967295), option=orjson.OPT_SERIALIZE_NUMPY),
            b"4294967295",
        )

    def test_numpy_scalar_uint64(self):
        self.assertEqual(
            orjson.dumps(numpy.uint64(0), option=orjson.OPT_SERIALIZE_NUMPY), b"0"
        )
        self.assertEqual(
            orjson.dumps(
                numpy.uint64(18446744073709551615), option=orjson.OPT_SERIALIZE_NUMPY
            ),
            b"18446744073709551615",
        )

    def test_numpy_scalar_float32(self):
        self.assertEqual(
            orjson.dumps(numpy.float32(1.0), option=orjson.OPT_SERIALIZE_NUMPY), b"1.0"
        )

    def test_numpy_scalar_float64(self):
        self.assertEqual(
            orjson.dumps(numpy.float64(123.123), option=orjson.OPT_SERIALIZE_NUMPY),
            b"123.123",
        )

    def test_numpy_bool(self):
        self.assertEqual(
            orjson.dumps(
                {"a": numpy.bool_(True), "b": numpy.bool_(False)},
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'{"a":true,"b":false}',
        )

    def test_numpy_datetime(self):
        self.assertEqual(
            orjson.dumps(
                {
                    "year": numpy.datetime64("2021"),
                    "month": numpy.datetime64("2021-01"),
                    "day": numpy.datetime64("2021-01-01"),
                    "hour": numpy.datetime64("2021-01-01T00"),
                    "minute": numpy.datetime64("2021-01-01T00:00"),
                    "second": numpy.datetime64("2021-01-01T00:00:00"),
                    "milli": numpy.datetime64("2021-01-01T00:00:00.172"),
                    "micro": numpy.datetime64("2021-01-01T00:00:00.172576"),
                    "nano": numpy.datetime64("2021-01-01T00:00:00.172576789"),
                },
                option=orjson.OPT_SERIALIZE_NUMPY,
            ),
            b'{"year":"2021-01-01T00:00:00","month":"2021-01-01T00:00:00","day":"2021-01-01T00:00:00","hour":"2021-01-01T00:00:00","minute":"2021-01-01T00:00:00","second":"2021-01-01T00:00:00","milli":"2021-01-01T00:00:00.172000","micro":"2021-01-01T00:00:00.172576","nano":"2021-01-01T00:00:00.172576"}',
        )

    def test_numpy_datetime_naive_utc(self):
        self.assertEqual(
            orjson.dumps(
                {
                    "year": numpy.datetime64("2021"),
                    "month": numpy.datetime64("2021-01"),
                    "day": numpy.datetime64("2021-01-01"),
                    "hour": numpy.datetime64("2021-01-01T00"),
                    "minute": numpy.datetime64("2021-01-01T00:00"),
                    "second": numpy.datetime64("2021-01-01T00:00:00"),
                    "milli": numpy.datetime64("2021-01-01T00:00:00.172"),
                    "micro": numpy.datetime64("2021-01-01T00:00:00.172576"),
                    "nano": numpy.datetime64("2021-01-01T00:00:00.172576789"),
                },
                option=orjson.OPT_SERIALIZE_NUMPY | orjson.OPT_NAIVE_UTC,
            ),
            b'{"year":"2021-01-01T00:00:00+00:00","month":"2021-01-01T00:00:00+00:00","day":"2021-01-01T00:00:00+00:00","hour":"2021-01-01T00:00:00+00:00","minute":"2021-01-01T00:00:00+00:00","second":"2021-01-01T00:00:00+00:00","milli":"2021-01-01T00:00:00.172000+00:00","micro":"2021-01-01T00:00:00.172576+00:00","nano":"2021-01-01T00:00:00.172576+00:00"}',
        )

    def test_numpy_datetime_naive_utc_utc_z(self):
        self.assertEqual(
            orjson.dumps(
                {
                    "year": numpy.datetime64("2021"),
                    "month": numpy.datetime64("2021-01"),
                    "day": numpy.datetime64("2021-01-01"),
                    "hour": numpy.datetime64("2021-01-01T00"),
                    "minute": numpy.datetime64("2021-01-01T00:00"),
                    "second": numpy.datetime64("2021-01-01T00:00:00"),
                    "milli": numpy.datetime64("2021-01-01T00:00:00.172"),
                    "micro": numpy.datetime64("2021-01-01T00:00:00.172576"),
                    "nano": numpy.datetime64("2021-01-01T00:00:00.172576789"),
                },
                option=orjson.OPT_SERIALIZE_NUMPY
                | orjson.OPT_NAIVE_UTC
                | orjson.OPT_UTC_Z,
            ),
            b'{"year":"2021-01-01T00:00:00Z","month":"2021-01-01T00:00:00Z","day":"2021-01-01T00:00:00Z","hour":"2021-01-01T00:00:00Z","minute":"2021-01-01T00:00:00Z","second":"2021-01-01T00:00:00Z","milli":"2021-01-01T00:00:00.172000Z","micro":"2021-01-01T00:00:00.172576Z","nano":"2021-01-01T00:00:00.172576Z"}',
        )

    def test_numpy_datetime_omit_microseconds(self):
        self.assertEqual(
            orjson.dumps(
                {
                    "year": numpy.datetime64("2021"),
                    "month": numpy.datetime64("2021-01"),
                    "day": numpy.datetime64("2021-01-01"),
                    "hour": numpy.datetime64("2021-01-01T00"),
                    "minute": numpy.datetime64("2021-01-01T00:00"),
                    "second": numpy.datetime64("2021-01-01T00:00:00"),
                    "milli": numpy.datetime64("2021-01-01T00:00:00.172"),
                    "micro": numpy.datetime64("2021-01-01T00:00:00.172576"),
                    "nano": numpy.datetime64("2021-01-01T00:00:00.172576789"),
                },
                option=orjson.OPT_SERIALIZE_NUMPY | orjson.OPT_OMIT_MICROSECONDS,
            ),
            b'{"year":"2021-01-01T00:00:00","month":"2021-01-01T00:00:00","day":"2021-01-01T00:00:00","hour":"2021-01-01T00:00:00","minute":"2021-01-01T00:00:00","second":"2021-01-01T00:00:00","milli":"2021-01-01T00:00:00","micro":"2021-01-01T00:00:00","nano":"2021-01-01T00:00:00"}',
        )

    def test_numpy_repeated(self):
        data = numpy.array([[[1, 2], [3, 4], [5, 6], [7, 8]]], numpy.int64)
        for _ in range(0, 3):
            self.assertEqual(
                orjson.dumps(
                    data,
                    option=orjson.OPT_SERIALIZE_NUMPY,
                ),
                b"[[[1,2],[3,4],[5,6],[7,8]]]",
            )
