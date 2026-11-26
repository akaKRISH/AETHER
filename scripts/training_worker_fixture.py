"""Independent standard-library consumer of the synthetic training wire format."""

import hashlib
import json
import re
import sys

BACKEND = "aether.synthetic-training-worker/1"
PINS = {"manifest_sha256", "base_sha256", "train_sha256", "validation_sha256"}
FIELDS = PINS | {
    "schema", "run_kind", "backend", "settings", "resource_caps",
    "planned_optimizer_steps", "cancel_before_or_at_step",
}


def unique_fields(pairs):
    fields = {}
    for key, value in pairs:
        if key in fields:
            raise ValueError("duplicate job field")
        fields[key] = value
    return fields


def replay(raw):
    job = json.loads(raw, object_pairs_hook=unique_fields)
    if (
        not isinstance(job, dict) or job.keys() != FIELDS
        or job["schema"] != "aether.training-worker-job/1"
        or job["run_kind"] != "synthetic" or job["backend"] != BACKEND
        or not all(isinstance(job[key], str) and re.fullmatch(r"[0-9a-f]{64}", job[key]) for key in PINS)
    ):
        raise ValueError("only the reviewed synthetic job is supported")
    steps = job["planned_optimizer_steps"]
    settings = job["settings"]
    every = settings.get("checkpoint_every_steps") if isinstance(settings, dict) else None
    cancel = job["cancel_before_or_at_step"]
    if (
        type(steps) is not int or not 1 <= steps <= 16_384
        or type(every) is not int or not 1 <= every <= steps
        or (cancel is not None and (type(cancel) is not int or not 0 <= cancel <= steps))
    ):
        raise ValueError("invalid synthetic schedule")
    canonical = json.dumps(job, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False)
    job_hash = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
    checkpoints = [
        {
            "step": step, "job_sha256": job_hash, "synthetic": True,
            "receipt_sha256": hashlib.sha256(f"{BACKEND}/{job_hash}/{step}".encode()).hexdigest(),
        }
        for step in range(1, steps + 1)
        if (step % every == 0 or step == steps) and (cancel is None or step < cancel)
    ]
    return {
        "job_sha256": job_hash, "checkpoints": checkpoints, "synthetic": True,
        "outcome": "completed" if cancel is None else "cancelled", "actual_weight_updates": 0,
    }


if __name__ == "__main__":
    try:
        raw = sys.stdin.buffer.read(65_537)
        if len(raw) > 65_536:
            raise ValueError("job exceeds 64 KiB")
        print(json.dumps(replay(raw)))
    except (ValueError, RecursionError) as error:
        raise SystemExit(str(error)) from error
