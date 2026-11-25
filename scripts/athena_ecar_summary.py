"""Summarize a frozen Athena synthetic development export."""

import itertools
import json
import math
from pathlib import Path
import random
import statistics
import sys

data = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8-sig"))
rows = data["outcomes"]
keys = ("family", "seed", "cap")
pairs = {}
for row in rows:
    assert row["used"]["wall_ms"] <= row["cap"]
    assert row["false_verified"] <= row["verified"] <= row["total_claims"]
    key = tuple(row[field] for field in keys)
    arms = pairs.setdefault(key, {})
    assert row["mode"] not in arms
    arms[row["mode"]] = row


def percent(row):
    return row["verified_utility"] * 100 / row["total_weight"]


def aggregate(mode):
    group = [pair[mode] for pair in pairs.values()]
    sums = {key: sum(row[key] for row in group) for key in (
        "verified", "false_verified", "rejected", "total_claims", "unresolved_high_impact",
        "replays", "replay_passes", "failed_attempts",
    )}
    true_verified = sums["verified"] - sums["false_verified"]
    used = sum(row["used"]["wall_ms"] for row in group)
    return sums | {
        "mean_utility_percent": statistics.mean(map(percent, group)),
        "false_verification_rate": sums["false_verified"] / sums["verified"] if sums["verified"] else None,
        "coverage": (sums["verified"] + sums["rejected"]) / sums["total_claims"],
        "used_simulated_ms": used,
        "budget_utilization": used / sum(row["cap"] for row in group),
        "simulated_ms_per_true_verified": used / true_verified if true_verified else None,
    }


def contrast(left, right):
    grouped = sorted((key[1], percent(arms[left]) - percent(arms[right]))
                     for key, arms in pairs.items())
    blocks = [statistics.mean(score for _, score in entries)
              for _, entries in itertools.groupby(grouped, key=lambda item: item[0])]
    stddev = statistics.stdev(blocks)
    rng = random.Random(617)
    boot = sorted(statistics.mean(rng.choices(blocks, k=len(blocks))) for _ in range(2000))
    return {
        "mean_points": statistics.mean(blocks),
        "seed_block_sd": stddev,
        "bootstrap_95": [boot[49], boot[1949]],
        "pilot_seed_blocks": len(blocks),
        "sample_size_at_5_points": max(30, math.ceil((1.96 + 0.84)**2 * stddev**2 / 25)),
        "by_family": {
            family: statistics.mean(percent(arms[left]) - percent(arms[right])
                                    for key, arms in pairs.items() if key[0] == family)
            for family in sorted({key[0] for key in pairs})
        },
        "wins": sum(score > 0 for _, score in grouped),
        "ties": sum(score == 0 for _, score in grouped),
        "losses": sum(score < 0 for _, score in grouped),
    }


print(json.dumps({
    "cases": len(rows), "host_wall_us": data["wall_us"], "host_scheduler_ns": data["scheduler_ns"],
    "arms": {mode: aggregate(mode) for mode in ("FixedFlat", "AdaptiveFlat", "FixedGraph", "Ecar")},
    "primary_ecar_vs_adaptive_flat": contrast("Ecar", "AdaptiveFlat"),
    "secondary_fixed_graph_vs_fixed_flat": contrast("FixedGraph", "FixedFlat"),
    "secondary_adaptive_flat_vs_fixed_flat": contrast("AdaptiveFlat", "FixedFlat"),
}, indent=2))
