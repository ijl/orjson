import unittest

import orjson
import pytest

try:
    import torch
    import numpy as np
    HAVE_PYTORCH = True
except ImportError:
    HAVE_PYTORCH = False

@pytest.mark.skipif(not HAVE_PYTORCH, reason="pytorch not installed")
class PyTorchTests(unittest.TestCase):
    def test_tensor_1d(self):
        """
        torch.Tensor, 1-dimensional
        """
        tensor = torch.tensor([1, 2, 3])
        self.assertEqual(orjson.dumps(tensor, option=orjson.OPT_SERIALIZE_NUMPY), b'[1,2,3]')

    def test_tensor_2d(self):
        """
        torch.Tensor, 2-dimensional
        """
        tensor = torch.tensor([[1, 2], [3, 4]])
        self.assertEqual(orjson.dumps(tensor, option=orjson.OPT_SERIALIZE_NUMPY), b'[[1,2],[3,4]]')

    def test_tensor_float(self):
        """
        torch.Tensor, float
        """
        tensor = torch.tensor([1.1, 2.2, 3.3])
        self.assertEqual(orjson.dumps(tensor, option=orjson.OPT_SERIALIZE_NUMPY), b'[1.1,2.2,3.3]')

    def test_tensor_bool(self):
        """
        torch.Tensor, bool
        """
        tensor = torch.tensor([True, False, True])
        self.assertEqual(orjson.dumps(tensor, option=orjson.OPT_SERIALIZE_NUMPY), b'[true,false,true]')

    def test_tensor_empty(self):
        """
        torch.Tensor, empty
        """
        tensor = torch.tensor([])
        self.assertEqual(orjson.dumps(tensor, option=orjson.OPT_SERIALIZE_NUMPY), b'[]')

    def test_tensor_without_numpy_opt(self):
        """
        torch.Tensor without OPT_SERIALIZE_NUMPY
        """
        tensor = torch.tensor([1, 2, 3])
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(tensor)

    def test_tensor_requires_grad(self):
        """
        torch.Tensor with requires_grad=True
        """
        tensor = torch.tensor([1., 2., 3.], requires_grad=True)
        self.assertEqual(orjson.dumps(tensor, option=orjson.OPT_SERIALIZE_NUMPY), b'[1.0,2.0,3.0]')

    def test_tensor_on_gpu(self):
        """
        torch.Tensor on GPU if available
        """
        if not torch.cuda.is_available():
            self.skipTest("CUDA not available")
        tensor = torch.tensor([1, 2, 3]).cuda()
        self.assertEqual(orjson.dumps(tensor, option=orjson.OPT_SERIALIZE_NUMPY), b'[1,2,3]')

    def test_tensor_on_gpu_and_requires_grad(self):
        """
        torch.Tensor on GPU if available AND requires_grad=True
        """
        if not torch.cuda.is_available():
            self.skipTest("CUDA not available")
        tensor = torch.tensor([1., 2., 3.], requires_grad=True).cuda()
        self.assertEqual(orjson.dumps(tensor, option=orjson.OPT_SERIALIZE_NUMPY), b'[1.0,2.0,3.0]')

    def test_tensor_zero_dim(self):
        """
        Test 0-dimensional tensors are properly serialized as scalar values
        """
        # Test float scalar tensor
        tensor_float = torch.tensor(0.03)
        self.assertEqual(orjson.dumps(tensor_float, option=orjson.OPT_SERIALIZE_NUMPY), b'0.03')

        # Test int scalar tensor
        tensor_int = torch.tensor(42)
        self.assertEqual(orjson.dumps(tensor_int, option=orjson.OPT_SERIALIZE_NUMPY), b'42')

        # Test in a nested structure
        data = {
            "scalar_float": torch.tensor(0.03),
            "scalar_int": torch.tensor(42),
            "array": torch.tensor([1, 2, 3]),
        }
        self.assertEqual(
            orjson.dumps(data, option=orjson.OPT_SERIALIZE_NUMPY),
            b'{"scalar_float":0.03,"scalar_int":42,"array":[1,2,3]}'
        )

    def test_tensor_special_values(self):
        """
        Test that special values (nan, inf) are properly serialized
        """
        # Test nan
        tensor_nan = torch.tensor(float('nan'))
        self.assertEqual(orjson.dumps(tensor_nan, option=orjson.OPT_SERIALIZE_NUMPY), b'NaN')

        # Test inf
        tensor_inf = torch.tensor(float('inf'))
        self.assertEqual(orjson.dumps(tensor_inf, option=orjson.OPT_SERIALIZE_NUMPY), b'Infinity')
        tensor_neg_inf = torch.tensor(float('-inf'))
        self.assertEqual(orjson.dumps(tensor_neg_inf, option=orjson.OPT_SERIALIZE_NUMPY), b'-Infinity')

        # Test in a nested structure
        data = {
            "nan": torch.tensor(float('nan')),
            "inf": torch.tensor(float('inf')),
            "neg_inf": torch.tensor(float('-inf')),
            "mixed": torch.tensor([1.0, float('nan'), float('inf'), float('-inf')]),
        }
        self.assertEqual(
            orjson.dumps(data, option=orjson.OPT_SERIALIZE_NUMPY),
            b'{"nan":NaN,"inf":Infinity,"neg_inf":-Infinity,"mixed":[1.0,NaN,Infinity,-Infinity]}'
        )