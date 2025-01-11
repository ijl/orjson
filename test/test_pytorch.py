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