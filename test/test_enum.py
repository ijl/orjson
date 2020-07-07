# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import datetime
import enum
import unittest

import orjson


class StrEnum(str, enum.Enum):
    AAA = "aaa"


class IntEnum(int, enum.Enum):
    ONE = 1


class IntEnumEnum(enum.IntEnum):
    ONE = 1


class IntFlagEnum(enum.IntFlag):
    ONE = 1


class FlagEnum(enum.Flag):
    ONE = 1


class AutoEnum(enum.auto):
    A = "a"


class FloatEnum(float, enum.Enum):
    ONE = 1.1


class Custom:
    def __init__(self, val):
        self.val = val


def default(obj):
    if isinstance(obj, Custom):
        return obj.val
    raise TypeError


class UnspecifiedEnum(enum.Enum):
    A = "a"
    B = 1
    C = FloatEnum.ONE
    D = {"d": IntEnum.ONE}
    E = Custom("c")
    F = datetime.datetime(1970, 1, 1)


class EnumTests(unittest.TestCase):
    def test_cannot_subclass(self):
        """
        enum.Enum cannot be subclassed

        obj->ob_type->ob_base will always be enum.EnumMeta
        """
        with self.assertRaises(TypeError):

            class Subclass(StrEnum):
                B = "b"

    def test_arbitrary_enum(self):
        self.assertEqual(orjson.dumps(UnspecifiedEnum.A), b'"a"')
        self.assertEqual(orjson.dumps(UnspecifiedEnum.B), b"1")
        self.assertEqual(orjson.dumps(UnspecifiedEnum.C), b"1.1")
        self.assertEqual(orjson.dumps(UnspecifiedEnum.D), b'{"d":1}')

    def test_custom_enum(self):
        self.assertEqual(orjson.dumps(UnspecifiedEnum.E, default=default), b'"c"')

    def test_enum_options(self):
        self.assertEqual(
            orjson.dumps(UnspecifiedEnum.F, option=orjson.OPT_NAIVE_UTC),
            b'"1970-01-01T00:00:00+00:00"',
        )

    def test_int_enum(self):
        self.assertEqual(orjson.dumps(IntEnum.ONE), b"1")

    def test_intenum_enum(self):
        self.assertEqual(orjson.dumps(IntEnumEnum.ONE), b"1")

    def test_intflag_enum(self):
        self.assertEqual(orjson.dumps(IntFlagEnum.ONE), b"1")

    def test_flag_enum(self):
        self.assertEqual(orjson.dumps(FlagEnum.ONE), b"1")

    def test_auto_enum(self):
        self.assertEqual(orjson.dumps(AutoEnum.A), b'"a"')

    def test_float_enum(self):
        self.assertEqual(orjson.dumps(FloatEnum.ONE), b"1.1")

    def test_str_enum(self):
        self.assertEqual(orjson.dumps(StrEnum.AAA), b'"aaa"')

    def test_bool_enum(self):
        with self.assertRaises(TypeError):

            class BoolEnum(bool, enum.Enum):
                TRUE = True

    def test_non_str_keys_enum(self):
        self.assertEqual(
            orjson.dumps({StrEnum.AAA: 1}, option=orjson.OPT_NON_STR_KEYS), b'{"aaa":1}'
        )
        self.assertEqual(
            orjson.dumps({IntEnum.ONE: 1}, option=orjson.OPT_NON_STR_KEYS), b'{"1":1}'
        )


class EnumPassthroughTests(unittest.TestCase):
    def test_passthrough_int_enum(self):
        """
        int, float, int- & float-derived Enums should not be unaffected by the passthrough flag
        """
        self.assertEqual(
            orjson.dumps(IntEnumEnum.ONE, option=orjson.OPT_PASSTHROUGH_ENUM), b"1"
        )

    def test_passthrough_int_flag_enum(self):
        """
        int, float, int- & float-derived Enums should not be unaffected by the passthrough flag
        """
        self.assertEqual(
            orjson.dumps(IntFlagEnum.ONE, option=orjson.OPT_PASSTHROUGH_ENUM), b"1"
        )

    def test_passthrough_flag_enum(self):
        """
        FlagEnum is a subclass of Enum meaning it has the following metaclass=EnumMeta.
        It should be affected by the passthrough.
        """
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(FlagEnum.ONE, option=orjson.OPT_PASSTHROUGH_ENUM)

    def test_passthrough_enum_callable(self):
        """
        names is a subclass of Enum meaning it has the following metaclass=EnumMeta.
        It should be affected by the passthrough.
        """
        names = enum.Enum("names", ("john", "alice"))
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(names.alice, option=orjson.OPT_PASSTHROUGH_ENUM)

    def test_passthrough_enum_inherit(self):
        class FooEnum(enum.Enum):
            name = "alice"

        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(FooEnum.name, option=orjson.OPT_PASSTHROUGH_ENUM)

    def test_passthrough_default_enum_default(self):
        def default(obj):
            return str(obj)

        names = enum.Enum("names", ("john", "alice"))
        self.assertEqual(
            orjson.dumps(
                names.alice, option=orjson.OPT_PASSTHROUGH_ENUM, default=default
            ),
            b'"names.alice"',
        )
        self.assertEqual(
            orjson.dumps(
                names.john, option=orjson.OPT_PASSTHROUGH_ENUM, default=default
            ),
            b'"names.john"',
        )
