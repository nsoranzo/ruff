from typing import overload


@overload
def foo(i: int) -> "int":
    ...


@overload
def foo(i: "str") -> "str":
    ...


def foo(i):
    return i
