from collections.abc import Iterable, Iterator
from itertools import islice

def _fill(l: list, i: int, e, it: Iterator):
    while e is not None:
        l[i] = e
        i += 1
        e = next(it, None)

def _merge(l: list, i: int, it1: Iterable, it2: Iterable):
    it1, it2 = iter(it1), iter(it2)
    e1 = next(it1, None)
    e2 = next(it2, None)
    while True:
        if e1 is None:
            return _fill(l, i, e2, it2)
        elif e2 is None:
            return _fill(l, i, e1, it1)
        elif e1 < e2:
            l[i] = e1
            e1 = next(it1, None)
        else:
            l[i] = e2
            e2 = next(it2, None)
        i += 1

def _merge_sort_impl(l: list, buf: list, lo: int, hi: int) -> list:
    if lo + 1 >= hi:
        return l
    mid = (lo + hi) // 2
    ll = _merge_sort_impl(l, buf, lo, mid)
    lr = _merge_sort_impl(l, buf, mid, hi)
    if ll is not lr:
        if ll is l:
            buf[lo:mid] = islice(ll, lo, mid)
        else:
            buf[mid:hi] = islice(lr, mid, hi)
    elif ll is l:
        l, buf = buf, l
    _merge(l, lo, islice(buf, lo, mid), islice(buf, mid, hi))
    return l

def merge_sort(l: list) -> list:
    return _merge_sort_impl(l, l.copy(), 0, len(l))

def main():
    l = [3, 2, 4, 0, 1, 6, 5]
    l = merge_sort(l)
    print(l)

if __name__ == "__main__":
    main()
