# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import re
import uuid
import unittest

import orjson


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

    def test_all_ways_to_create_uuid_behave_equivalently(self):
        # Note that according to the docstring for the uuid.UUID class, all the
        # forms below are equivalent -- they end up with the same value for
        # `self.int`, which is all that really matters
        uuids = [
            uuid.UUID("{12345678-1234-5678-1234-567812345678}"),
            uuid.UUID("12345678123456781234567812345678"),
            uuid.UUID("urn:uuid:12345678-1234-5678-1234-567812345678"),
            uuid.UUID(bytes=b"\x12\x34\x56\x78" * 4),
            uuid.UUID(
                bytes_le=b"\x78\x56\x34\x12\x34\x12\x78\x56"
                + b"\x12\x34\x56\x78\x12\x34\x56\x78"
            ),
            uuid.UUID(fields=(0x12345678, 0x1234, 0x5678, 0x12, 0x34, 0x567812345678)),
            uuid.UUID(int=0x12345678123456781234567812345678),
        ]
        result = orjson.dumps(uuids, option=orjson.OPT_SERIALIZE_UUID)
        canonical_uuids = ['"%s"' % str(u) for u in uuids]
        serialized = ("[%s]" % ",".join(canonical_uuids)).encode("utf8")
        self.assertEqual(result, serialized)

    def test_serialize_natively_equivalent_to_str(self):
        uuid_ = uuid.uuid4()
        self.assertEqual(
            orjson.dumps([uuid_], option=orjson.OPT_SERIALIZE_UUID),
            orjson.dumps([uuid_], default=str),
        )

    def test_does_not_serialize_without_opt(self):
        with self.assertRaises(orjson.JSONEncodeError):
            _ = orjson.dumps([uuid.uuid4()])

    def test_serializes_correctly_with_leading_zeroes(self):
        instance = uuid.UUID(int=0x00345678123456781234567812345678)
        self.assertEqual(
            orjson.dumps(instance, option=orjson.OPT_SERIALIZE_UUID),
            ('"%s"' % str(instance)).encode("utf8"),
        )

    def test_all_uuid_creation_functions_create_serializable_uuids(self):
        all_versioned_uuids = [
            uuid.uuid1(),
            uuid.uuid3(uuid.NAMESPACE_DNS, "python.org"),
            uuid.uuid4(),
            uuid.uuid5(uuid.NAMESPACE_DNS, "python.org"),
        ]
        serialized = orjson.dumps(all_versioned_uuids, option=orjson.OPT_SERIALIZE_UUID)
        # Ensure that all the creator functions produce UUID strings that match
        # our expected 8-4-4-4-12 hexadecimal format
        assert re.match(
            rb'\[("[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",?){4}]',
            serialized,
        )
