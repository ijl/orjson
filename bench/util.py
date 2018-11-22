# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import os
import json
import lzma


dirname = os.path.join(os.path.dirname(__file__), '../data')


def read_fixture_str(filename):
    if filename.endswith('.xz'):
        with lzma.open(os.path.join(dirname, filename), 'r') as fileh:
            return fileh.read()
    else:
        with open(os.path.join(dirname, filename), 'r') as fileh:
            return fileh.read()


def read_fixture_obj(filename):
    return json.loads(read_fixture_str(filename))
