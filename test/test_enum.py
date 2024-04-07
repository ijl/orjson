# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import datetime
import enum

import pytest

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


class TestEnum:
    def test_cannot_subclass(self):
        """
        enum.Enum cannot be subclassed

        obj->ob_type->ob_base will always be enum.EnumMeta
        """
        with pytest.raises(TypeError):

            class Subclass(StrEnum):  # type: ignore
                B = "b"

    @pytest.mark.parametrize("enum_value, expected", [
        (UnspecifiedEnum.A, b'"a"'),
        (UnspecifiedEnum.B, b"1"),
        (UnspecifiedEnum.C, b"1.1"),
        (UnspecifiedEnum.D, b'{"d":1}'),
        (IntEnumEnum.ONE, b"1"),
        (IntFlagEnum.ONE, b"1"),
        (FlagEnum.ONE, b"1"),
        (AutoEnum.A, b'"a"'),
        (FloatEnum.ONE, b"1.1"),
        (StrEnum.AAA, b'"aaa"'),
    ])
    def test_enum_serialization(self, enum_value, expected):
        assert orjson.dumps(enum_value) == expected


    def test_custom_enum(self):
        assert orjson.dumps(UnspecifiedEnum.E, default=default) == b'"c"'

    def test_enum_options(self):
        assert (
            orjson.dumps(UnspecifiedEnum.F, option=orjson.OPT_NAIVE_UTC)
            == b'"1970-01-01T00:00:00+00:00"'
        )

    def test_bool_enum(self):
        with pytest.raises(TypeError):

            class BoolEnum(bool, enum.Enum):  # type: ignore
                TRUE = True

    def test_non_str_keys_enum(self):
        assert (
            orjson.dumps({StrEnum.AAA: 1}, option=orjson.OPT_NON_STR_KEYS)
            == b'{"aaa":1}'
        )
        assert (
            orjson.dumps({IntEnum.ONE: 1}, option=orjson.OPT_NON_STR_KEYS) == b'{"1":1}'
        )
