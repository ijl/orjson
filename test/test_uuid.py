# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import uuid
import unittest


class UUIDTests(unittest.TestCase):
    def test_uuid_immutable(self):
        """
        UUID objects are immutable
        """
        val = uuid.uuid4()
        with self.assertRaises(TypeError):
            val.int = 1
        with self.assertRaises(TypeError):
            val.int = None

    def test_uuid_int(self):
        """
        UUID.int is a 128-bit integer
        """
        val = uuid.UUID("7202d115-7ff3-4c81-a7c1-2a1f067b1ece")
        self.assertIsInstance(val.int, int)
        self.assertTrue(val.int >= 2 ** 64)
        self.assertTrue(val.int < 2 ** 128)
        self.assertEqual(val.int, 151546616840194781678008611711208857294)

    def test_uuid_overflow(self):
        """
        UUID.int can't trigger errors in _PyLong_AsByteArray
        """
        with self.assertRaises(ValueError):
            uuid.UUID(int=2 ** 128)
        with self.assertRaises(ValueError):
            uuid.UUID(int=-1)
