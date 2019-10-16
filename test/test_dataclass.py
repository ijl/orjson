# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest
import pytest
import sys

from dataclasses import dataclass, field
from typing import Optional

import orjson


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
    c: float


class Datasubclass(Dataclass1):
    number: float


class DataclassTests(unittest.TestCase):
    def test_dataclass(self):
        """
        dumps() dataclass
        """
        obj = Dataclass1("a", 1, None)
        self.assertEqual(orjson.dumps(obj), b'{"name":"a","number":1,"sub":null}')

    def test_dataclass_recursive(self):
        """
        dumps() dataclass recursive
        """
        obj = Dataclass1("a", 1, Dataclass1("b", 2, None))
        self.assertEqual(
            orjson.dumps(obj),
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

    def test_dataclass_default(self):
        """
        dumps() dataclass default
        """
        obj = Dataclass2()
        self.assertEqual(orjson.dumps(obj), b'{"name":"?"}')

    def test_dataclass_types(self):
        """
        dumps() dataclass types
        """
        obj = Dataclass3("a", 1, {"a": "b"}, True, 1.1, [1, 2], (3, 4))
        self.assertEqual(
            orjson.dumps(obj),
            b'{"a":"a","b":1,"c":{"a":"b"},"d":true,"e":1.1,"f":[1,2],"g":[3,4]}',
        )

    def test_dataclass_metadata(self):
        """
        dumps() dataclass metadata
        """
        obj = Dataclass4("a", 1, 2.1)
        self.assertEqual(orjson.dumps(obj), b'{"a":"a","b":1,"c":2.1}')

    def test_dataclass_subclass(self):
        """
        dumps() dataclass subclass
        """
        obj = Datasubclass("a", 1.0, None)
        self.assertEqual(orjson.dumps(obj), b'{"name":"a","number":1.0,"sub":null}')
