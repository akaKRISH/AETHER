"""Summarize an Athena export under the selected analysis protocol."""

import hashlib
import itertools
import json
import math
from pathlib import Path
import random
import statistics
import sys

MODES = ("FixedFlat", "AdaptiveFlat", "FixedGraph", "Ecar")
PROTOCOL = Path(__file__).resolve().parents[1] / "docs/research/ATH_02B_ECAR_PROTOCOL.md"


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def digest(value):
    return isinstance(value, str) and len(value) == 64 and all(c in "0123456789abcdef" for c in value)


def check(condition, message):
    if not condition:
        raise ValueError(message)


def integer(value, lower, upper):
    return type(value) is int and lower <= value <= upper


def grid(config, phase):
    check(config["version"] == "aether.ecar-config/2" and config["phase"] == phase,
          "configuration version or phase mismatch")
    first, count = config["first_seed"], config["seed_count"]
    check(integer(first, 0, 2**32 - 1) and integer(count, 30 if phase == "heldout" else 2, 256)
          and first + count < 2**32, "invalid seed range for analysis")
    caps, scenarios = config["caps"], config["scenarios"]
    check(1 <= len(caps) <= 6 and all(integer(cap, 8, 240) for cap in caps)
          and len(set(caps)) == len(caps), "invalid or duplicate caps")
    check(1 <= len(scenarios) <= 8, "invalid family count")
    families = {}
    for scenario in scenarios:
        family, weights = scenario["id"], scenario["weights"]
        check(isinstance(family, str) and family and family not in families,
              "invalid or duplicate family ID")
        check(1 <= len(weights) <= 16 and all(integer(weight, 1, 100) for weight in weights),
              "invalid claim weights")
        families[family] = weights
    return families, set(itertools.product(families, range(first, first + count), caps))


def validate_report(data, config_bytes, phase, approval=None, approval_digest=None):
    check(data["version"] == "aether.ecar-diagnostics/2" and data["phase"] == phase
          and data["scope"] == f"synthetic {phase}", "report version or phase mismatch")
    pins = {"protocol_sha256": sha256(PROTOCOL.read_bytes()), "config_sha256": sha256(config_bytes)}
    if phase == "heldout":
        check(approval is not None and sha256(approval) == approval_digest,
              "approval manifest digest mismatch")
        manifest = json.loads(approval)
        check(set(manifest) == {"version", "analysis_sha256", "runner_sha256", *pins}
              and manifest["version"] == "aether.ecar-approval/1", "invalid approval manifest")
        for name, expected in (pins | {"analysis_sha256": sha256(Path(__file__).read_bytes())}).items():
            check(manifest[name] == expected, f"approval {name} mismatch")
        check(digest(manifest["runner_sha256"]), "invalid approved runner digest")
        pins["runner_sha256"] = manifest["runner_sha256"]
    check(digest(data["runner_sha256"]), "invalid report runner digest")
    for name, expected in (pins | {"approval_sha256": approval_digest}).items():
        check(data[name] == expected, f"report {name} mismatch")
    return grid(json.loads(config_bytes), phase)


def validate_metrics(row, weights):
    count, total_weight = len(weights), sum(weights)
    verified, rejected = row["verified"], row["rejected"]
    used = row["used"]
    check(integer(row["total_claims"], count, count) and integer(row["total_weight"], total_weight, total_weight)
          and integer(verified, 0, count) and integer(rejected, 0, count - verified)
          and integer(row["false_verified"], 0, verified)
          and integer(row["verified_utility"], 0, total_weight)
          and integer(row["unresolved_high_impact"], 0, min(count - verified - rejected,
                                                           sum(w >= 9 for w in weights)))
          and integer(row["replays"], 0, count)
          and integer(row["replay_passes"], 0, row["replays"])
          and integer(used["wall_ms"], 0, row["cap"]) and integer(used["attempts"], 0, 48)
          and integer(row["failed_attempts"], 0, min(count, used["attempts"]))
          and all(integer(used[name], 0, 0) for name in ("model_tokens", "paid_microusd", "output_bytes")),
          "invalid metric denominator, count or budget")


def read_pairs(data, expected=None):
    pairs = {}
    for row in data["outcomes"]:
        key = (row["family"], row["seed"], row["cap"])
        mode = row["mode"]
        if expected is None:
            check(row["used"]["wall_ms"] <= row["cap"]
                  and row["false_verified"] <= row["verified"] <= row["total_claims"], "invalid legacy outcome")
        else:
            families, cases = expected
            check(type(key[1]) is int and type(key[2]) is int and key in cases
                  and mode in MODES, "unexpected case or arm")
            validate_metrics(row, families[key[0]])
        arms = pairs.setdefault(key, {})
        check(mode not in arms, "duplicate case and arm")
        arms[mode] = row
    if expected is not None:
        check(pairs.keys() == expected[1], "missing cases")
        check(all(arms.keys() == set(MODES) for arms in pairs.values()), "unmatched case arms")
    return pairs


def percent(row):
    return row["verified_utility"] * 100 / row["total_weight"]


def aggregate(pairs, mode):
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


def contrast(pairs, left, right):
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


def main(args):
    match args:
        case [report]:
            data = json.loads(Path(report).read_text(encoding="utf-8-sig"))
            check(data["version"] == "aether.ecar-diagnostics/1" and data["scope"] == "synthetic development",
                  "the archival command accepts version 1 development reports only")
            pairs = read_pairs(data)
        case ["--development-v2", report, config] | ["--heldout", report, config, *_]:
            phase = "heldout" if args[0] == "--heldout" else "development"
            check(len(args) == (5 if phase == "heldout" else 3), "wrong number of analysis arguments")
            approval, approved = (Path(args[3]).read_bytes(), args[4]) if phase == "heldout" else (None, None)
            data = json.loads(Path(report).read_text(encoding="utf-8-sig"))
            pairs = read_pairs(data, validate_report(data, Path(config).read_bytes(), phase, approval, approved))
        case _:
            raise ValueError("usage: <report> | --development-v2 <report> <config> | "
                             "--heldout <report> <config> <manifest> <approved-digest>")

    summary = {
        "cases": len(data["outcomes"]), "host_wall_us": data["wall_us"], "host_scheduler_ns": data["scheduler_ns"],
        "arms": {mode: aggregate(pairs, mode) for mode in MODES},
        "primary_ecar_vs_adaptive_flat": contrast(pairs, "Ecar", "AdaptiveFlat"),
        "secondary_fixed_graph_vs_fixed_flat": contrast(pairs, "FixedGraph", "FixedFlat"),
        "secondary_adaptive_flat_vs_fixed_flat": contrast(pairs, "AdaptiveFlat", "FixedFlat"),
    }
    if data["version"] == "aether.ecar-diagnostics/2":
        summary["provenance"] = {name: data[name] for name in (
            "version", "phase", "protocol_sha256", "config_sha256", "runner_sha256", "approval_sha256",
        )}
        for value in summary.values():
            if isinstance(value, dict) and "pilot_seed_blocks" in value:
                value["seed_blocks"] = value.pop("pilot_seed_blocks")
                del value["sample_size_at_5_points"]
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    try:
        main(sys.argv[1:])
    except (KeyError, OSError, TypeError, ValueError) as error:
        sys.exit(f"Analysis rejected: {error}")
