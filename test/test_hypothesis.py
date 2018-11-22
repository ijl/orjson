# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import pytest
from hypothesis import given, strategies as st

import orjson


# min, max: RFC 7159
st_int = st.integers(min_value=-(2**53)+1, max_value=(2**53)-1)
st_floats = st.floats(min_value=-(2**53)+1, max_value=(2**53)-1)

# st.floats would be nice, but then we need pytest.approx, which doesn't work with eg. text
st_json = st.recursive(st.booleans() | st.none() | st_int  # | st_floats  # st.text() |
                       , lambda children: st.lists(children) | st.dictionaries(st.text(), children))


@given(st_floats)
def test_floats(xs):
    assert orjson.loads(orjson.dumps(xs)) == pytest.approx(
        xs)  # fails when abs=0.05


@given(st.text())
def test_text(xs):
    assert orjson.loads(orjson.dumps(xs)) == xs


@given(st.booleans())
def test_bool(xs):
    assert orjson.loads(orjson.dumps(xs)) == xs


@given(st.none())
def test_none(xs):
    assert orjson.loads(orjson.dumps(xs)) == xs


@given(st.lists(st_int))
def test_list_integers(lst):
    assert orjson.loads(orjson.dumps(lst)) == lst


@given(st.lists(st.floats(min_value=-(2**53)+1, max_value=(2**53)-1)))
def test_list_floats(lst):
    assert orjson.loads(orjson.dumps(lst)) == pytest.approx(lst)


@given(st.lists(st.text()))
def test_list_text(lst):
    assert orjson.loads(orjson.dumps(lst)) == lst


@given(st.lists(st.one_of(st_int, st_floats)))
def test_list_mixed(lst):
    assert orjson.loads(orjson.dumps(lst)) == pytest.approx(lst)


@given(st_json)
def test_json_obj(j_obj):
    assert orjson.loads(orjson.dumps(j_obj)) == j_obj
