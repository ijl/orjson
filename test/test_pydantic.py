import unittest
import orjson
from typing import Optional

from pydantic import BaseModel

class Model1(BaseModel):
    hi: str
    number: int
    sub: Optional[int]

class Model2(BaseModel):
    bye: str
    previous: Model1


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