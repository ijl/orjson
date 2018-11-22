# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import orjson


class ApiTests(unittest.TestCase):

    def test_version(self):
        """
        __version__
        """
        self.assertRegex(orjson.__version__, r'^\d+\.\d+(\.\d+)?$')
