import orjson


class C:
    c: "C"

    def __del__(self):
        orjson.loads('"' + "a" * 10000 + '"')


def test_reentrant():
    c = C()
    c.c = c
    del c

    orjson.loads("[" + "[]," * 1000 + "[]]")
