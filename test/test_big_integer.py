# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import pytest

import orjson


class TestBigIntegerTests:

    def test_big_integer_dumps(self):
        """
        Big integers are serialized as strings
        """
        
        with pytest.raises(TypeError):
            orjson.dumps(100000000000000000001)

        with pytest.raises(TypeError):
            orjson.dumps(-100000000000000000001)

        assert orjson.dumps(100000000000000000001, option=orjson.OPT_BIG_INTEGER) == b'100000000000000000001'
        assert orjson.dumps(-100000000000000000001, option=orjson.OPT_BIG_INTEGER) == b'-100000000000000000001'

    def test_big_integer_loads(self):
        """
        Big integers are deserialized as integers
        """        

        assert orjson.loads(b'10000000000000000000') == 10000000000000000000
        assert orjson.loads(b'-10000000000000000000') == -10000000000000000000

        assert orjson.loads(b'100000000000000000001', option=orjson.OPT_BIG_INTEGER) == 100000000000000000001
        assert orjson.loads(b'-100000000000000000001', option=orjson.OPT_BIG_INTEGER) == -100000000000000000001

    def test_big_integers_dict_key_dumps(self):
        """
        Big integers as dict keys are serialized as strings
        """
        
        with pytest.raises(TypeError):
            orjson.dumps({100000000000000000001: True}, option=orjson.OPT_NON_STR_KEYS)

        with pytest.raises(TypeError):
            orjson.dumps({-100000000000000000001: True}, option=orjson.OPT_NON_STR_KEYS)

        assert orjson.dumps({100000000000000000001: True}, option=orjson.OPT_NON_STR_KEYS | orjson.OPT_BIG_INTEGER) == b'{"100000000000000000001":true}'
        assert orjson.dumps({-100000000000000000001: True}, option=orjson.OPT_NON_STR_KEYS | orjson.OPT_BIG_INTEGER) == b'{"-100000000000000000001":true}'


    def test_big_integers_dict_key_loads(self):
        """
        Big integers as dict keys are deserialized as integers
        """
        assert orjson.loads(b'{"10000000000000000000":true}') == {"10000000000000000000": True}
        assert orjson.loads(b'{"-10000000000000000000":true}') == {"-10000000000000000000": True}

        assert orjson.loads(b'{"100000000000000000001":true}') == {"100000000000000000001": True}
        assert orjson.loads(b'{"-100000000000000000001":true}') == {"-100000000000000000001": True}

        with pytest.raises(orjson.JSONDecodeError):
            orjson.loads(b'{10000000000000000000:true}')

    