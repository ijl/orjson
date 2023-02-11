# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import orjson

from .util import read_fixture_obj


class TestAppendNewline:
    def test_dumps_newline(self):
        """
        dumps() OPT_APPEND_NEWLINE
        """
        assert orjson.dumps([], option=orjson.OPT_APPEND_NEWLINE) == b"[]\n"

    def test_twitter_newline(self):
        """
        loads(),dumps() twitter.json OPT_APPEND_NEWLINE
        """
        val = read_fixture_obj("twitter.json.xz")
        assert orjson.loads(orjson.dumps(val, option=orjson.OPT_APPEND_NEWLINE)) == val

    def test_canada(self):
        """
        loads(), dumps() canada.json OPT_APPEND_NEWLINE
        """
        val = read_fixture_obj("canada.json.xz")
        assert orjson.loads(orjson.dumps(val, option=orjson.OPT_APPEND_NEWLINE)) == val

    def test_citm_catalog_newline(self):
        """
        loads(), dumps() citm_catalog.json OPT_APPEND_NEWLINE
        """
        val = read_fixture_obj("citm_catalog.json.xz")
        assert orjson.loads(orjson.dumps(val, option=orjson.OPT_APPEND_NEWLINE)) == val

    def test_github_newline(self):
        """
        loads(), dumps() github.json OPT_APPEND_NEWLINE
        """
        val = read_fixture_obj("github.json.xz")
        assert orjson.loads(orjson.dumps(val, option=orjson.OPT_APPEND_NEWLINE)) == val
