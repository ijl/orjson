# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import json
import lzma
import os
import unittest

import orjson


dirname = os.path.dirname(__file__)


def fixture_fileh(filename):
    return lzma.open(os.path.join(dirname, "../data", filename), "r")


def read_fixture(filename):
    with fixture_fileh(filename) as fileh:
        return fileh.read().decode("utf-8")


class FixtureTests(unittest.TestCase):
    def test_twitter(self):
        """
        loads(),dumps() twitter.json
        """
        val = read_fixture("twitter.json.xz")
        read = json.loads(val)
        orjson.dumps(read)

    def test_canada(self):
        """
        loads(), dumps() canada.json
        """
        val = read_fixture("canada.json.xz")
        read = orjson.loads(val)
        orjson.dumps(read)

    def test_citm_catalog(self):
        """
        loads(), dumps() citm_catalog.json
        """
        val = read_fixture("citm_catalog.json.xz")
        read = orjson.loads(val)
        orjson.dumps(read)

    def test_blns(self):
        """
        loads() blns.json JSONDecodeError

        https://github.com/minimaxir/big-list-of-naughty-strings
        """
        val = read_fixture("blns.txt.xz")
        for line in val.split("\n"):
            if line and not line.startswith("#"):
                with self.assertRaises(json.decoder.JSONDecodeError):
                    _ = orjson.loads('"' + val + '"')
