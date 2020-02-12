# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest
import uuid
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

import orjson


class AnEnum(Enum):
    ONE = 1
    TWO = 2


@dataclass
class Dataclass1:
    name: str
    number: int
    sub: Optional["Dataclass1"]


@dataclass
class Dataclass2:
    name: Optional[str] = field(default="?")


@dataclass
class Dataclass3:
    a: str
    b: int
    c: dict
    d: bool
    e: float
    f: list
    g: tuple


@dataclass
class Dataclass4:
    a: str = field()
    b: int = field(metadata={"unrelated": False})
    c: float = 1.1


@dataclass
class Datasubclass(Dataclass1):
    additional: bool


@dataclass
class Slotsdataclass:
    __slots__ = ("a", "b")
    a: str
    b: int


@dataclass
class Defaultdataclass:
    a: uuid.UUID
    b: AnEnum


@dataclass
class UnsortedDataclass:
    c: int
    b: int
    a: int


class DataclassTests(unittest.TestCase):
    def test_dataclass_error(self):
        """
        dumps() dataclass error without option
        """
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(Dataclass1("a", 1, None))

    def test_dataclass(self):
        """
        dumps() dataclass
        """
        obj = Dataclass1("a", 1, None)
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_DATACLASS),
            b'{"name":"a","number":1,"sub":null}',
        )

    def test_dataclass_recursive(self):
        """
        dumps() dataclass recursive
        """
        obj = Dataclass1("a", 1, Dataclass1("b", 2, None))
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_DATACLASS),
            b'{"name":"a","number":1,"sub":{"name":"b","number":2,"sub":null}}',
        )

    def test_dataclass_circular(self):
        """
        dumps() dataclass circular
        """
        obj1 = Dataclass1("a", 1, None)
        obj2 = Dataclass1("b", 2, obj1)
        obj1.sub = obj2
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(obj1)

    def test_dataclass_default_arg(self):
        """
        dumps() dataclass default arg
        """
        obj = Dataclass2()
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_DATACLASS), b'{"name":"?"}'
        )

    def test_dataclass_types(self):
        """
        dumps() dataclass types
        """
        obj = Dataclass3("a", 1, {"a": "b"}, True, 1.1, [1, 2], (3, 4))
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_DATACLASS),
            b'{"a":"a","b":1,"c":{"a":"b"},"d":true,"e":1.1,"f":[1,2],"g":[3,4]}',
        )

    def test_dataclass_metadata(self):
        """
        dumps() dataclass metadata
        """
        obj = Dataclass4("a", 1, 2.1)
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_DATACLASS),
            b'{"a":"a","b":1,"c":2.1}',
        )

    def test_dataclass_classvar(self):
        """
        dumps() dataclass class variable
        """
        obj = Dataclass4("a", 1)
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_DATACLASS),
            b'{"a":"a","b":1,"c":1.1}',
        )

    def test_dataclass_subclass(self):
        """
        dumps() dataclass subclass
        """
        obj = Datasubclass("a", 1, None, False)
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_DATACLASS),
            b'{"name":"a","number":1,"sub":null,"additional":false}',
        )

    def test_dataclass_slots(self):
        """
        dumps() dataclass with __slots__
        """
        obj = Slotsdataclass("a", 1)
        assert "__dict__" not in dir(obj)
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_DATACLASS), b'{"a":"a","b":1}'
        )

    def test_dataclass_default(self):
        """
        dumps() dataclass with default
        """

        def default(__obj):
            if isinstance(__obj, uuid.UUID):
                return str(__obj)
            elif isinstance(__obj, Enum):
                return __obj.value

        obj = Defaultdataclass(
            uuid.UUID("808989c0-00d5-48a8-b5c4-c804bf9032f2"), AnEnum.ONE
        )
        self.assertEqual(
            orjson.dumps(obj, default=default, option=orjson.OPT_SERIALIZE_DATACLASS),
            b'{"a":"808989c0-00d5-48a8-b5c4-c804bf9032f2","b":1}',
        )

    def test_dataclass_sort(self):
        """
        OPT_SORT_KEYS has no effect on dataclasses
        """
        obj = UnsortedDataclass(1, 2, 3)
        self.assertEqual(
            orjson.dumps(
                obj, option=orjson.OPT_SERIALIZE_DATACLASS | orjson.OPT_SORT_KEYS
            ),
            b'{"c":1,"b":2,"a":3}',
        )
