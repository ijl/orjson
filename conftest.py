import pytest
import sys

is_python35 = sys.version_info.minor == 5


def pytest_ignore_collect(path, config):
    if is_python35 and str(path).endswith("test_typeddict.py"):
        return True
