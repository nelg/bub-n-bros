"""Sorting helpers that reproduce Python 2's "compare anything" ordering.

Python 2 could sort lists such as [(points, player), ...] even when two
entries had equal points: it then compared the player objects themselves
(by type name and memory address).  Python 3 raises TypeError instead.

Use py2key as the sort key wherever such mixed or object-containing
tuples are sorted:

    lst.sort(key=py2key)
    best = min(lst, key=py2key)
"""


def py2key(x):
    if x is None:
        return (0,)
    if isinstance(x, (bool, int, float)):
        return (1, x)
    if isinstance(x, tuple):
        return (2, 'tuple', tuple([py2key(item) for item in x]))
    if isinstance(x, list):
        return (2, 'list', tuple([py2key(item) for item in x]))
    if isinstance(x, (str, bytes)):
        return (2, type(x).__name__, x)
    # arbitrary objects: by type name, then by identity, like Python 2 did
    return (2, type(x).__name__, id(x))
