# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import os
import lzma

from flask import Flask
import orjson

app = Flask(__name__)

filename = os.path.join(os.path.dirname(__file__), "..", "data", "twitter.json.xz")

with lzma.open(filename, "r") as fileh:
    DATA = orjson.loads(fileh.read())


@app.route("/")
def root():
    data = orjson.dumps(DATA)
    return app.response_class(
        response=data, status=200, mimetype="application/json; charset=utf-8"
    )
