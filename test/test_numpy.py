# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import orjson
import pytest

try:
    import numpy
except ImportError:
    numpy = None


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

    def test_numpy_array_d0(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(numpy.int32(1), option=orjson.OPT_SERIALIZE_NUMPY)

    def test_numpy_array_fotran(self):
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

    def test_numpy_array_unsupported_dtype(self):
        array = numpy.array([[1, 2], [3, 4]], numpy.int8)
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY)
        self.assertEqual(
            orjson.dumps(
                array, default=numpy_default, option=orjson.OPT_SERIALIZE_NUMPY
            ),
            orjson.dumps(array.tolist()),
        )

    def test_numpy_array_d1(self):
        array = numpy.array([1])
        self.assertEqual(
            orjson.loads(orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY,)),
            array.tolist(),
        )

    def test_numpy_array_d2(self):
        array = numpy.array([[1]])
        self.assertEqual(
            orjson.loads(orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY,)),
            array.tolist(),
        )

    def test_numpy_array_d3(self):
        array = numpy.array([[[1]]])
        self.assertEqual(
            orjson.loads(orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY,)),
            array.tolist(),
        )

    def test_numpy_array_d4(self):
        array = numpy.array([[[[1]]]])
        self.assertEqual(
            orjson.loads(orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY,)),
            array.tolist(),
        )

    def test_numpy_array_4_stride(self):
        array = numpy.random.rand(4, 4, 4, 4)
        self.assertEqual(
            orjson.loads(orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY,)),
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
            orjson.loads(orjson.dumps(array, option=orjson.OPT_SERIALIZE_NUMPY,)),
            array.tolist(),
        )
