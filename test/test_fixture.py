# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import pytest

import orjson

from .util import read_fixture_bytes, read_fixture_str


class TestFixture:
    def test_twitter(self):
        """
        loads(),dumps() twitter.json
        """
        val = read_fixture_str("twitter.json.xz")
        read = orjson.loads(val)
        assert orjson.loads(orjson.dumps(read)) == read

    def test_canada(self):
        """
        loads(), dumps() canada.json
        """
        val = read_fixture_str("canada.json.xz")
        read = orjson.loads(val)
        assert orjson.loads(orjson.dumps(read)) == read

    def test_citm_catalog(self):
        """
        loads(), dumps() citm_catalog.json
        """
        val = read_fixture_str("citm_catalog.json.xz")
        read = orjson.loads(val)
        assert orjson.loads(orjson.dumps(read)) == read

    def test_github(self):
        """
        loads(), dumps() github.json
        """
        val = read_fixture_str("github.json.xz")
        read = orjson.loads(val)
        assert orjson.loads(orjson.dumps(read)) == read

    def test_blns(self):
        """
        loads() blns.json JSONDecodeError

        https://github.com/minimaxir/big-list-of-naughty-strings
        """
        val = read_fixture_bytes("blns.txt.xz")
        for line in val.split(b"\n"):
            if line and not line.startswith(b"#"):
                with pytest.raises(orjson.JSONDecodeError):
                    _ = orjson.loads(b'"' + val + b'"')
