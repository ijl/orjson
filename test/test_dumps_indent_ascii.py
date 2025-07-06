# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import datetime
import json
import pytest

import orjson

from .util import needs_data, read_fixture_obj


def _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2) -> bytes:
    """
    Helper function to mimic json.dumps with options.
    """
    return json.dumps(
        obj,
        indent=(2 if opt_indent_2 else None),
        ensure_ascii=bool(opt_ensure_ascii),
        # Minify separators if not indenting.
        separators=(",", ":") if not opt_indent_2 else None,
    ).encode("utf-8")


@needs_data
class TestIndentedAsciiOutput:
    @pytest.mark.parametrize(
        "opt_indent_2", [0, orjson.OPT_INDENT_2], ids=["no_indent", "indent_2"]
    )
    @pytest.mark.parametrize(
        "opt_ensure_ascii",
        [0, orjson.OPT_ENSURE_ASCII],
        ids=["ensure_ascii_false", "ensure_ascii_true"],
    )
    def test_basic_equivalent(self, opt_indent_2, opt_ensure_ascii):
        obj = {"a": "b", "c": {"d": True}, "e": [1, 2]}

        assert orjson.dumps(
            obj, option=opt_ensure_ascii | opt_indent_2
        ) == _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2)

    @pytest.mark.parametrize(
        "opt_indent_2", [0, orjson.OPT_INDENT_2], ids=["no_indent", "indent_2"]
    )
    @pytest.mark.parametrize(
        "opt_ensure_ascii",
        [0, orjson.OPT_ENSURE_ASCII],
        ids=["ensure_ascii_false", "ensure_ascii_true"],
    )
    def test_basic_equivalent_with_emojis(self, opt_indent_2, opt_ensure_ascii):
        obj = {"a": "🩷b", "c🍉": {"d": True}, "e": [1, 2]}

        assert orjson.dumps(
            obj, option=opt_ensure_ascii | opt_indent_2
        ) == _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2)

    @pytest.mark.parametrize(
        "opt_indent_2", [0, orjson.OPT_INDENT_2], ids=["no_indent", "indent_2"]
    )
    @pytest.mark.parametrize(
        "opt_ensure_ascii",
        [0, orjson.OPT_ENSURE_ASCII],
        ids=["ensure_ascii_false", "ensure_ascii_true"],
    )
    def test_basic_equivalent_with_emojis_and_nonascii(self, opt_indent_2, opt_ensure_ascii):
        obj = {"a": "🩷b", "c🍉": {"d": True}, "e_你好": [1, 2]}

        assert orjson.dumps(
            obj, option=opt_ensure_ascii | opt_indent_2
        ) == _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2)

    @pytest.mark.parametrize(
        "opt_indent_2", [0, orjson.OPT_INDENT_2], ids=["no_indent", "indent_2"]
    )
    @pytest.mark.parametrize(
        "opt_ensure_ascii",
        [0, orjson.OPT_ENSURE_ASCII],
        ids=["ensure_ascii_false", "ensure_ascii_true"],
    )
    def test_empty(self, opt_ensure_ascii, opt_indent_2):
        obj = [{}, [[[]]], {"key": []}]

        assert orjson.dumps(
            obj, option=opt_ensure_ascii | opt_indent_2
        ) == _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2)

    @pytest.mark.parametrize(
        "opt_indent_2", [0, orjson.OPT_INDENT_2], ids=["no_indent", "indent_2"]
    )
    @pytest.mark.parametrize(
        "opt_ensure_ascii",
        [0, orjson.OPT_ENSURE_ASCII],
        ids=["ensure_ascii_false", "ensure_ascii_true"],
    )
    def test_twitter_pretty(self, opt_ensure_ascii, opt_indent_2):
        obj = read_fixture_obj("twitter.json.xz")

        assert orjson.dumps(
            obj, option=opt_ensure_ascii | opt_indent_2
        ) == _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2)

    @pytest.mark.parametrize(
        "opt_indent_2", [0, orjson.OPT_INDENT_2], ids=["no_indent", "indent_2"]
    )
    @pytest.mark.parametrize(
        "opt_ensure_ascii",
        [0, orjson.OPT_ENSURE_ASCII],
        ids=["ensure_ascii_false", "ensure_ascii_true"],
    )
    def test_github_pretty(self, opt_ensure_ascii, opt_indent_2):
        obj = read_fixture_obj("github.json.xz")

        assert orjson.dumps(
            obj, option=opt_ensure_ascii | opt_indent_2
        ) == _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2)

    @pytest.mark.parametrize(
        "opt_indent_2", [0, orjson.OPT_INDENT_2], ids=["no_indent", "indent_2"]
    )
    @pytest.mark.parametrize(
        "opt_ensure_ascii",
        [0, orjson.OPT_ENSURE_ASCII],
        ids=["ensure_ascii_false", "ensure_ascii_true"],
    )
    def test_canada_pretty(self, opt_ensure_ascii, opt_indent_2):
        obj = read_fixture_obj("canada.json.xz")

        assert orjson.dumps(
            obj, option=opt_ensure_ascii | opt_indent_2
        ) == _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2)

    @pytest.mark.parametrize(
        "opt_indent_2", [0, orjson.OPT_INDENT_2], ids=["no_indent", "indent_2"]
    )
    @pytest.mark.parametrize(
        "opt_ensure_ascii",
        [0, orjson.OPT_ENSURE_ASCII],
        ids=["ensure_ascii_false", "ensure_ascii_true"],
    )
    def test_citm_catalog_pretty(self, opt_ensure_ascii, opt_indent_2):
        obj = read_fixture_obj("citm_catalog.json.xz")

        assert orjson.dumps(
            obj, option=opt_ensure_ascii | opt_indent_2
        ) == _json_dumps_encode_with_opts(obj, opt_ensure_ascii, opt_indent_2)
