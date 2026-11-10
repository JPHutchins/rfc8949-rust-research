"""Least-squares per-message-type slope and intercept of flash size for each (backend, profile)."""

import csv
import sys
from collections.abc import Iterable
from itertools import groupby
from operator import itemgetter
from typing import NamedTuple


class Fit(NamedTuple):
    backend: str
    profile: str
    intercept: float
    per_type: float
    flash_n32: int


def fit(rows: list[dict[str, str]]) -> Fit:
    points = [(int(r["message_count"]), int(r["flash"])) for r in rows if int(r["message_count"]) >= 1]
    mean_x = sum(x for x, _ in points) / len(points)
    mean_y = sum(y for _, y in points) / len(points)
    slope = sum((x - mean_x) * (y - mean_y) for x, y in points) / sum((x - mean_x) ** 2 for x, _ in points)
    by_n = dict(points)
    return Fit(rows[0]["backend"], rows[0]["profile"], mean_y - slope * mean_x, slope, by_n[32])


def fits(rows: Iterable[dict[str, str]]) -> list[Fit]:
    key = itemgetter("profile", "backend")
    return sorted(
        (fit(list(group)) for _, group in groupby(sorted(rows, key=key), key=key)),
        key=lambda f: (f.profile, f.per_type),
    )


if __name__ == "__main__":
    print("| backend | profile | fixed cost: fit intercept (B) | marginal B / message type | flash @ N=32 (B) |")
    print("|---|---|---:|---:|---:|")
    for f in fits(csv.DictReader(open(sys.argv[1]))):
        print(f"| {f.backend} | {'opt-level=z' if f.profile == 'release' else 'opt-level=s'} | {f.intercept:,.0f} | {f.per_type:,.0f} | {f.flash_n32:,} |")
