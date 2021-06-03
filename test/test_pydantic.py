import abc
import unittest
from typing import Optional, Sequence, Tuple, Union

import numpy as np
from pydantic import BaseModel

import orjson


class Model1(BaseModel):
    hi: str
    number: int
    sub: Optional[int]


class Model2(BaseModel):
    bye: str
    previous: Model1


class Shape(abc.ABC):
    @abc.abstractmethod
    def calculate_area(self) -> float:
        ...


class Circle(BaseModel):
    __slots__ = "radius"
    radius: float

    @staticmethod
    def __private_static_method(arg1: int, arg2: str):
        raise NotImplementedError

    @staticmethod
    def public_static_method(arg1: Sequence[Optional[Tuple[int, ...]]]):
        raise NotImplementedError

    def __private_instance_method(self, other: "Circle") -> Optional["Circle"]:
        raise NotImplementedError

    def calculate_area(self) -> float:
        raise NotImplementedError


class ConvexPolygon(Shape):
    @property
    @abc.abstractmethod
    def number_of_sides(self) -> int:
        ...


class RegularPolygon(ConvexPolygon, BaseModel):
    __slots__ = ("side_count", "side_length", "vertex_coordinates")
    side_count: int
    side_length: float
    vertex_coordinates: Optional[np.ndarray]

    @staticmethod
    def __private_static_method(arg1: int, arg2: str):
        raise NotImplementedError

    @staticmethod
    def public_static_method(arg1: Sequence[Optional[Tuple[int, ...]]]):
        raise NotImplementedError

    def __private_instance_method(
        self, other: "RegularPolygon"
    ) -> Optional["RegularPolygon"]:
        raise NotImplementedError

    @property
    def number_of_sides(self) -> int:
        return self.side_count

    def calculate_area(self) -> float:
        raise NotImplementedError

    class Config:
        arbitrary_types_allowed = True


class ComplexShape(Shape, BaseModel):
    __slots__ = ("inner_shapes", "name")
    inner_shapes: Sequence[Union["Circle", "RegularPolygon"]]
    name: str

    def calculate_area(self) -> float:
        raise NotImplementedError


class PydanticTests(unittest.TestCase):
    def test_basemodel(self):
        """
        dumps() pydantic basemodel
        """
        obj = Model1(hi="a", number=1, sub=None)
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_PYDANTIC),
            b'{"hi":"a","number":1,"sub":null}',
        )

    def test_recursive_basemodel(self):
        """
        dumps() pydantic basemodel with another basemodel as attribute
        """
        obj = Model1(hi="a", number=1, sub=None)
        obj2 = Model2(previous=obj, bye="lala")
        self.assertEqual(
            orjson.dumps(obj2, option=orjson.OPT_SERIALIZE_PYDANTIC),
            b'{"bye":"lala","previous":{"hi":"a","number":1,"sub":null}}',
        )

    def test_dataclass_with_abc1(self):
        """
        dumps() dataclass implementing an ABC
        """
        obj = RegularPolygon(side_count=8, side_length=12.345, vertex_coordinates=None)
        self.assertEqual(
            orjson.dumps(obj, option=orjson.OPT_SERIALIZE_PYDANTIC),
            b'{"side_count":8,"side_length":12.345,"vertex_coordinates":null}',
        )

    def test_dataclass_with_abc2(self):
        """
        dumps() dataclasses implementing a common ABC, nested within a dictionary.
        """
        shapes = {
            "octagon": RegularPolygon(
                side_count=8, side_length=12.345, vertex_coordinates=None
            ),
            "circle": Circle(radius=839.4871),
            "decagon": RegularPolygon(
                side_count=10, side_length=2.1112, vertex_coordinates=None
            ),
            "triangle": RegularPolygon(
                side_count=3, side_length=0.9090909, vertex_coordinates=None
            ),
        }
        self.assertEqual(
            orjson.dumps(shapes, option=orjson.OPT_SERIALIZE_PYDANTIC),
            b'{"octagon":{"side_count":8,"side_length":12.345,"vertex_coordinates":null},"circle":{"radius":839.4871},"decagon":{"side_count":10,"side_length":2.1112,"vertex_coordinates":null},"triangle":{"side_count":3,"side_length":0.9090909,"vertex_coordinates":null}}',
        )

    def test_dataclass_with_abc3(self):
        """
        dumps() dataclasses implementing a common ABC, nested within a another dataclass implementing a superclass ABC of the common ABC.
        """
        inner = (
            RegularPolygon(side_count=8, side_length=12.345, vertex_coordinates=None),
            Circle(radius=839.4871),
            RegularPolygon(side_count=10, side_length=2.1112, vertex_coordinates=None),
            RegularPolygon(
                side_count=3, side_length=0.9090909, vertex_coordinates=None
            ),
        )

        complex_shape = ComplexShape(inner_shapes=inner, name="weird shape")
        self.assertEqual(
            orjson.dumps(complex_shape, option=orjson.OPT_SERIALIZE_PYDANTIC),
            b'{"inner_shapes":[{"side_count":8,"side_length":12.345,"vertex_coordinates":null},{"radius":839.4871},{"side_count":10,"side_length":2.1112,"vertex_coordinates":null},{"side_count":3,"side_length":0.9090909,"vertex_coordinates":null}],"name":"weird shape"}',
        )
