# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import datetime
import sys
import unittest

import pytest
import pytz
from dateutil import tz

import orjson

try:
    import pendulum
except ImportError:
    pendulum = None  # type: ignore

if sys.version_info >= (3, 9):
    import zoneinfo


class DatetimeTests(unittest.TestCase):
    def test_datetime_naive(self):
        """
        datetime.datetime naive prints without offset
        """
        self.assertEqual(
            orjson.dumps([datetime.datetime(2000, 1, 1, 2, 3, 4, 123)]),
            b'["2000-01-01T02:03:04.000123"]',
        )

    def test_datetime_naive_utc(self):
        """
        datetime.datetime naive with opt assumes UTC
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
                option=orjson.OPT_NAIVE_UTC,
            ),
            b'["2000-01-01T02:03:04.000123+00:00"]',
        )

    def test_datetime_min(self):
        """
        datetime.datetime min range
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(datetime.MINYEAR, 1, 1, 0, 0, 0, 0)],
                option=orjson.OPT_NAIVE_UTC,
            ),
            b'["0001-01-01T00:00:00+00:00"]',
        )

    def test_datetime_max(self):
        """
        datetime.datetime max range
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(datetime.MAXYEAR, 12, 31, 23, 59, 50, 999999)],
                option=orjson.OPT_NAIVE_UTC,
            ),
            b'["9999-12-31T23:59:50.999999+00:00"]',
        )

    def test_datetime_three_digits(self):
        """
        datetime.datetime three digit year
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(312, 1, 1)],
                option=orjson.OPT_NAIVE_UTC,
            ),
            b'["0312-01-01T00:00:00+00:00"]',
        )

    def test_datetime_two_digits(self):
        """
        datetime.datetime two digit year
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(46, 1, 1)],
                option=orjson.OPT_NAIVE_UTC,
            ),
            b'["0046-01-01T00:00:00+00:00"]',
        )

    def test_datetime_tz_assume(self):
        """
        datetime.datetime tz with assume UTC uses tz
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018, 1, 1, 2, 3, 4, 0, tzinfo=tz.gettz("Asia/Shanghai")
                    )
                ],
                option=orjson.OPT_NAIVE_UTC,
            ),
            b'["2018-01-01T02:03:04+08:00"]',
        )

    def test_datetime_timezone_utc(self):
        """
        datetime.datetime UTC
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018, 6, 1, 2, 3, 4, 0, tzinfo=datetime.timezone.utc
                    )
                ]
            ),
            b'["2018-06-01T02:03:04+00:00"]',
        )

    def test_datetime_pytz_utc(self):
        """
        datetime.datetime UTC
        """
        self.assertEqual(
            orjson.dumps([datetime.datetime(2018, 6, 1, 2, 3, 4, 0, tzinfo=pytz.UTC)]),
            b'["2018-06-01T02:03:04+00:00"]',
        )

    @unittest.skipIf(
        sys.version_info < (3, 9) or sys.platform.startswith("win"),
        "zoneinfo not available",
    )
    def test_datetime_zoneinfo_positive(self):
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        1,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=zoneinfo.ZoneInfo("Asia/Shanghai"),
                    )
                ]
            ),
            b'["2018-01-01T02:03:04+08:00"]',
        )

    @unittest.skipIf(
        sys.version_info < (3, 9) or sys.platform.startswith("win"),
        "zoneinfo not available",
    )
    def test_datetime_zoneinfo_negative(self):
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        6,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=zoneinfo.ZoneInfo("America/New_York"),
                    )
                ]
            ),
            b'["2018-06-01T02:03:04-04:00"]',
        )

    @pytest.mark.skipif(pendulum is None, reason="pendulum install broken on win")
    def test_datetime_pendulum_utc(self):
        """
        datetime.datetime UTC
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(2018, 6, 1, 2, 3, 4, 0, tzinfo=pendulum.UTC)]
            ),
            b'["2018-06-01T02:03:04+00:00"]',
        )

    def test_datetime_arrow_positive(self):
        """
        datetime.datetime positive UTC
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018, 1, 1, 2, 3, 4, 0, tzinfo=tz.gettz("Asia/Shanghai")
                    )
                ]
            ),
            b'["2018-01-01T02:03:04+08:00"]',
        )

    def test_datetime_pytz_positive(self):
        """
        datetime.datetime positive UTC
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018, 1, 1, 2, 3, 4, 0, tzinfo=pytz.timezone("Asia/Shanghai")
                    )
                ]
            ),
            b'["2018-01-01T02:03:04+08:00"]',
        )

    @pytest.mark.skipif(pendulum is None, reason="pendulum install broken on win")
    def test_datetime_pendulum_positive(self):
        """
        datetime.datetime positive UTC
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        1,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=pendulum.timezone("Asia/Shanghai"),
                    )
                ]
            ),
            b'["2018-01-01T02:03:04+08:00"]',
        )

    def test_datetime_pytz_negative_dst(self):
        """
        datetime.datetime negative UTC DST
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018, 6, 1, 2, 3, 4, 0, tzinfo=pytz.timezone("America/New_York")
                    )
                ]
            ),
            b'["2018-06-01T02:03:04-04:00"]',
        )

    @pytest.mark.skipif(pendulum is None, reason="pendulum install broken on win")
    def test_datetime_pendulum_negative_dst(self):
        """
        datetime.datetime negative UTC DST
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        6,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=pendulum.timezone("America/New_York"),
                    )
                ]
            ),
            b'["2018-06-01T02:03:04-04:00"]',
        )

    def test_datetime_pytz_negative_non_dst(self):
        """
        datetime.datetime negative UTC non-DST
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        12,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=pytz.timezone("America/New_York"),
                    )
                ]
            ),
            b'["2018-12-01T02:03:04-05:00"]',
        )

    @pytest.mark.skipif(pendulum is None, reason="pendulum install broken on win")
    def test_datetime_pendulum_negative_non_dst(self):
        """
        datetime.datetime negative UTC non-DST
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        12,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=pendulum.timezone("America/New_York"),
                    )
                ]
            ),
            b'["2018-12-01T02:03:04-05:00"]',
        )

    def test_datetime_partial_hour(self):
        """
        datetime.datetime UTC offset partial hour
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        12,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=pytz.timezone("Australia/Adelaide"),
                    )
                ]
            ),
            b'["2018-12-01T02:03:04+10:30"]',
        )

    def test_datetime_pytz_partial_hour(self):
        """
        datetime.datetime UTC offset partial hour
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        12,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=pytz.timezone("Australia/Adelaide"),
                    )
                ]
            ),
            b'["2018-12-01T02:03:04+10:30"]',
        )

    @pytest.mark.skipif(pendulum is None, reason="pendulum install broken on win")
    def test_datetime_pendulum_partial_hour(self):
        """
        datetime.datetime UTC offset partial hour
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2018,
                        12,
                        1,
                        2,
                        3,
                        4,
                        0,
                        tzinfo=pendulum.timezone("Australia/Adelaide"),
                    )
                ]
            ),
            b'["2018-12-01T02:03:04+10:30"]',
        )

    @pytest.mark.skipif(pendulum is None, reason="pendulum install broken on win")
    def test_datetime_partial_second_pendulum_supported(self):
        """
        datetime.datetime UTC offset round seconds

        https://tools.ietf.org/html/rfc3339#section-5.8
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        1937,
                        1,
                        1,
                        12,
                        0,
                        27,
                        87,
                        tzinfo=pendulum.timezone("Europe/Amsterdam"),
                    )
                ]
            ),
            b'["1937-01-01T12:00:27.000087+00:20"]',
        )

    def test_datetime_partial_second_pytz(self):
        """
        datetime.datetime UTC offset round seconds

        https://tools.ietf.org/html/rfc3339#section-5.8
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        1937,
                        1,
                        1,
                        12,
                        0,
                        27,
                        87,
                        tzinfo=pytz.timezone("Europe/Amsterdam"),
                    )
                ]
            ),
            b'["1937-01-01T12:00:27.000087+00:20"]',
        )

    def test_datetime_partial_second_dateutil(self):
        """
        datetime.datetime UTC offset round seconds

        https://tools.ietf.org/html/rfc3339#section-5.8
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        1937, 1, 1, 12, 0, 27, 87, tzinfo=tz.gettz("Europe/Amsterdam")
                    )
                ]
            ),
            b'["1937-01-01T12:00:27.000087+00:20"]',
        )

    def test_datetime_microsecond_max(self):
        """
        datetime.datetime microsecond max
        """
        self.assertEqual(
            orjson.dumps(datetime.datetime(2000, 1, 1, 0, 0, 0, 999999)),
            b'"2000-01-01T00:00:00.999999"',
        )

    def test_datetime_microsecond_min(self):
        """
        datetime.datetime microsecond min
        """
        self.assertEqual(
            orjson.dumps(datetime.datetime(2000, 1, 1, 0, 0, 0, 1)),
            b'"2000-01-01T00:00:00.000001"',
        )

    def test_datetime_omit_microseconds(self):
        """
        datetime.datetime OPT_OMIT_MICROSECONDS
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
                option=orjson.OPT_OMIT_MICROSECONDS,
            ),
            b'["2000-01-01T02:03:04"]',
        )

    def test_datetime_omit_microseconds_naive(self):
        """
        datetime.datetime naive OPT_OMIT_MICROSECONDS
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
                option=orjson.OPT_NAIVE_UTC | orjson.OPT_OMIT_MICROSECONDS,
            ),
            b'["2000-01-01T02:03:04+00:00"]',
        )

    def test_time_omit_microseconds(self):
        """
        datetime.time OPT_OMIT_MICROSECONDS
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.time(2, 3, 4, 123)], option=orjson.OPT_OMIT_MICROSECONDS
            ),
            b'["02:03:04"]',
        )

    def test_datetime_utc_z_naive_omit(self):
        """
        datetime.datetime naive OPT_UTC_Z
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
                option=orjson.OPT_NAIVE_UTC
                | orjson.OPT_UTC_Z
                | orjson.OPT_OMIT_MICROSECONDS,
            ),
            b'["2000-01-01T02:03:04Z"]',
        )

    def test_datetime_utc_z_naive(self):
        """
        datetime.datetime naive OPT_UTC_Z
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)],
                option=orjson.OPT_NAIVE_UTC | orjson.OPT_UTC_Z,
            ),
            b'["2000-01-01T02:03:04.000123Z"]',
        )

    def test_datetime_utc_z_without_tz(self):
        """
        datetime.datetime naive OPT_UTC_Z
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.datetime(2000, 1, 1, 2, 3, 4, 123)], option=orjson.OPT_UTC_Z
            ),
            b'["2000-01-01T02:03:04.000123"]',
        )

    def test_datetime_utc_z_with_tz(self):
        """
        datetime.datetime naive OPT_UTC_Z
        """
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        2000, 1, 1, 0, 0, 0, 1, tzinfo=datetime.timezone.utc
                    )
                ],
                option=orjson.OPT_UTC_Z,
            ),
            b'["2000-01-01T00:00:00.000001Z"]',
        )
        self.assertEqual(
            orjson.dumps(
                [
                    datetime.datetime(
                        1937, 1, 1, 12, 0, 27, 87, tzinfo=tz.gettz("Europe/Amsterdam")
                    )
                ],
                option=orjson.OPT_UTC_Z,
            ),
            b'["1937-01-01T12:00:27.000087+00:20"]',
        )

    @pytest.mark.skipif(pendulum is None, reason="pendulum install broken on win")
    def test_datetime_roundtrip(self):
        """
        datetime.datetime parsed by pendulum
        """
        obj = datetime.datetime(2000, 1, 1, 0, 0, 0, 1, tzinfo=datetime.timezone.utc)
        serialized = orjson.dumps(obj).decode("utf-8").replace('"', "")
        parsed = pendulum.parse(serialized)
        for attr in ("year", "month", "day", "hour", "minute", "second", "microsecond"):
            self.assertEqual(getattr(obj, attr), getattr(parsed, attr))


class DateTests(unittest.TestCase):
    def test_date(self):
        """
        datetime.date
        """
        self.assertEqual(orjson.dumps([datetime.date(2000, 1, 13)]), b'["2000-01-13"]')

    def test_date_min(self):
        """
        datetime.date MINYEAR
        """
        self.assertEqual(
            orjson.dumps([datetime.date(datetime.MINYEAR, 1, 1)]), b'["0001-01-01"]'
        )

    def test_date_max(self):
        """
        datetime.date MAXYEAR
        """
        self.assertEqual(
            orjson.dumps([datetime.date(datetime.MAXYEAR, 12, 31)]), b'["9999-12-31"]'
        )

    def test_date_three_digits(self):
        """
        datetime.date three digit year
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.date(312, 1, 1)],
            ),
            b'["0312-01-01"]',
        )

    def test_date_two_digits(self):
        """
        datetime.date two digit year
        """
        self.assertEqual(
            orjson.dumps(
                [datetime.date(46, 1, 1)],
            ),
            b'["0046-01-01"]',
        )


class TimeTests(unittest.TestCase):
    def test_time(self):
        """
        datetime.time
        """
        self.assertEqual(
            orjson.dumps([datetime.time(12, 15, 59, 111)]), b'["12:15:59.000111"]'
        )
        self.assertEqual(orjson.dumps([datetime.time(12, 15, 59)]), b'["12:15:59"]')

    def test_time_tz(self):
        """
        datetime.time with tzinfo error
        """
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(
                [datetime.time(12, 15, 59, 111, tzinfo=tz.gettz("Asia/Shanghai"))]
            )

    def test_time_microsecond_max(self):
        """
        datetime.time microsecond max
        """
        self.assertEqual(
            orjson.dumps(datetime.time(0, 0, 0, 999999)), b'"00:00:00.999999"'
        )

    def test_time_microsecond_min(self):
        """
        datetime.time microsecond min
        """
        self.assertEqual(orjson.dumps(datetime.time(0, 0, 0, 1)), b'"00:00:00.000001"')


class DateclassPassthroughTests(unittest.TestCase):
    def test_passthrough_datetime(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(
                datetime.datetime(1970, 1, 1), option=orjson.OPT_PASSTHROUGH_DATETIME
            )

    def test_passthrough_date(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(
                datetime.date(1970, 1, 1), option=orjson.OPT_PASSTHROUGH_DATETIME
            )

    def test_passthrough_time(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(
                datetime.time(12, 0, 0), option=orjson.OPT_PASSTHROUGH_DATETIME
            )

    def test_passthrough_datetime_default(self):
        def default(obj):
            return obj.strftime("%a, %d %b %Y %H:%M:%S GMT")

        self.assertEqual(
            orjson.dumps(
                datetime.datetime(1970, 1, 1),
                option=orjson.OPT_PASSTHROUGH_DATETIME,
                default=default,
            ),
            b'"Thu, 01 Jan 1970 00:00:00 GMT"',
        )
