"""Measure or draft against an owned local-only Ollama process on Windows."""

import argparse
from contextlib import contextmanager
import ctypes
from ctypes import wintypes
import hashlib
import http.client
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import threading
import time

MODEL = "qwen3.5:4b"
PROMPT = "Write one short sentence explaining why an independent replay is useful."
MIB = 1024 * 1024
ROOT = Path(__file__).resolve().parents[1]


class MemoryStatus(ctypes.Structure):
    _fields_ = [("length", wintypes.DWORD), ("load", wintypes.DWORD),
                *[(name, ctypes.c_ulonglong) for name in
                  ("total", "available", "total_page", "available_page",
                   "total_virtual", "available_virtual", "available_extended")]]


def available_ram():
    status = MemoryStatus()
    status.length = ctypes.sizeof(status)
    if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status)):
        raise ctypes.WinError()
    return status.available


def cpu_counts():
    idle, kernel, user = (ctypes.c_ulonglong() for _ in range(3))
    if not ctypes.windll.kernel32.GetSystemTimes(*(ctypes.byref(item) for item in (idle, kernel, user))):
        raise ctypes.WinError()
    return idle.value, kernel.value + user.value


def gpu():
    tool = Path(os.environ["SystemRoot"]) / "System32/nvidia-smi.exe"
    result = subprocess.check_output(
        [tool, "--query-gpu=name,driver_version,memory.total,memory.free,memory.used,utilization.gpu",
         "--format=csv,noheader,nounits"],
        text=True, timeout=3, creationflags=subprocess.CREATE_NO_WINDOW,
    ).strip()
    name, driver, total, free, used, utilization = (part.strip() for part in result.rsplit(",", 5))
    return dict(name=name, driver=driver, total_mib=int(total), free_mib=int(free),
                used_mib=int(used), utilization_percent=int(utilization))


def request(port, path, data=None):
    with_request = http.client.HTTPConnection("127.0.0.1", port, timeout=60)
    with_request.request(
        "POST" if data is not None else "GET", path, None if data is None else json.dumps(data),
        {"Content-Type": "application/json"},
    )
    response = with_request.getresponse()
    if response.status != 200:
        message = response.read(4096).decode(errors="replace")
        with_request.close()
        raise RuntimeError(f"{path} HTTP {response.status}: {message}")
    return with_request, response


def query(port, path, data=None):
    connection, response = request(port, path, data)
    try:
        body = response.read(2 * MIB + 1)
        if len(body) > 2 * MIB:
            raise RuntimeError("oversized metadata response")
        return json.loads(body)
    finally:
        connection.close()


def stop_tree(process):
    if process.poll() is None:
        subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"],
                       capture_output=True, check=False, timeout=8)
        if process.poll() is None:
            process.kill()
        process.wait(timeout=8)


@contextmanager
def local_server():
    if os.name != "nt":
        raise RuntimeError("the Athena calibration harness is for Windows")
    if available_ram() < 2048 * MIB or gpu()["free_mib"] < 4608:
        raise RuntimeError("insufficient available RAM or VRAM for the installed local model")
    executable = shutil.which("ollama")
    if executable is None:
        raise RuntimeError("Ollama is unavailable")
    installed = Path(os.environ.get("OLLAMA_MODELS") or Path.home() / ".ollama/models")
    relative = Path("manifests/registry.ollama.ai/library/qwen3.5/4b")
    manifest = (installed / relative).read_bytes()
    digest = hashlib.sha256(manifest).hexdigest()
    definition = json.loads(manifest)
    descriptors = [definition["config"], *definition["layers"]]
    if any("remote" in item["mediaType"] for item in descriptors):
        raise RuntimeError("remote model descriptors are ineligible")
    with tempfile.TemporaryDirectory(prefix="aether-athena-ollama-") as scratch:
        private = Path(scratch) / "models"
        (private / relative).parent.mkdir(parents=True)
        (private / relative).write_bytes(manifest)
        (private / "blobs").mkdir()
        for descriptor in descriptors:
            filename = descriptor["digest"].replace(":", "-", 1)
            source = installed / "blobs" / filename
            if source.stat().st_size != descriptor["size"]:
                raise RuntimeError("local model blob size mismatch")
            os.link(source, private / "blobs" / filename)
        with socket.socket() as reservation:
            reservation.bind(("127.0.0.1", 0))
            port = reservation.getsockname()[1]
        windows_environment = {
            "SYSTEMROOT", "SYSTEMDRIVE", "WINDIR", "PROGRAMFILES", "PROGRAMFILES(X86)",
            "PROGRAMW6432", "PATH", "PATHEXT", "APPDATA", "LOCALAPPDATA", "PROGRAMDATA",
            "USERPROFILE", "HOMEDRIVE", "HOMEPATH", "TEMP", "TMP", "COMSPEC",
        }
        env = {key: value for key, value in os.environ.items() if key.upper() in windows_environment}
        env.update(OLLAMA_HOST=f"127.0.0.1:{port}", OLLAMA_MODELS=str(private), OLLAMA_NO_CLOUD="1",
                   OLLAMA_NOPRUNE="1", OLLAMA_NUM_PARALLEL="1", OLLAMA_MAX_LOADED_MODELS="1")
        with (Path(scratch) / "server.log").open("wb") as log:
            daemon = subprocess.Popen([executable, "serve"], env=env, stdout=log, stderr=log,
                                      creationflags=subprocess.CREATE_NO_WINDOW)
            verified = False
            try:
                for _ in range(80):
                    if daemon.poll() is not None:
                        raise RuntimeError("owned Ollama process exited during startup")
                    try:
                        version = query(port, "/api/version")["version"]
                        break
                    except OSError:
                        time.sleep(0.1)
                else:
                    raise RuntimeError("owned Ollama endpoint did not start")
                try:
                    cloud = query(port, "/api/v1/cloud")
                except RuntimeError as error:
                    if "HTTP 404" not in str(error):
                        raise
                    cloud = query(port, "/api/status")["cloud"]
                models = query(port, "/api/tags")["models"]
                if cloud["disabled"] is not True or len(models) != 1:
                    raise RuntimeError("local-only checks failed")
                entry = models[0]
                if (entry["name"] != MODEL or entry["digest"] != digest
                        or entry.get("remote_host") or entry.get("remote_model")):
                    raise RuntimeError("local model identity was not confirmed")
                details = query(port, "/api/show", {"model": MODEL})
                if "completion" not in details["capabilities"] or query(port, "/api/ps")["models"]:
                    raise RuntimeError("expected a cold local completion model")
                verified = True
                yield port, daemon, dict(model=MODEL, manifest_digest=digest, version=version,
                                        cloud=cloud, details=details["details"],
                                        blob_bytes=sum(item["size"] for item in descriptors))
            finally:
                if verified and daemon.poll() is None:
                    try:
                        query(port, "/api/generate", {"model": MODEL, "keep_alive": 0})
                    except (OSError, RuntimeError):
                        pass
                stop_tree(daemon)


def measured_trial(port, context, kind):
    request_body = {"model": MODEL, "prompt": PROMPT, "stream": True, "think": False,
                    "keep_alive": "45s", "options": {"seed": 19, "temperature": 0,
                                                     "num_predict": 32, "num_ctx": context}}
    started = time.perf_counter()
    connection, response = request(port, "/api/generate", request_body)
    first, output, final = None, "", None
    try:
        for _ in range(100):
            line = response.readline(16385)
            if not line or len(line) > 16384:
                raise RuntimeError("incomplete or oversized generation stream")
            record = json.loads(line)
            if (record.get("error") or record.get("model") != MODEL
                    or record.get("remote_host") or record.get("remote_model")):
                raise RuntimeError("generation failed or switched away from the local model")
            if chunk := record.get("response"):
                if first is None:
                    first = (time.perf_counter() - started) * 1000
                output += chunk
            if record.get("done"):
                final = record
                break
        if final is None or first is None:
            raise RuntimeError("generation returned no complete visible text")
    finally:
        connection.close()
    counters = {key: final[key] for key in ("load_duration", "prompt_eval_count", "prompt_eval_duration",
                                            "eval_count", "eval_duration", "total_duration")}
    return dict(kind=kind, request=request_body, output=output, ttft_ms=first,
                elapsed_ms=(time.perf_counter() - started) * 1000, counters=counters,
                resident=query(port, "/api/ps")["models"])


def execute(args):
    with local_server() as (port, daemon, model):
        if args.command == "preflight":
            return model
        if args.command == "draft":
            executable = ROOT / "target/release/aether-lab.exe"
            env = os.environ | {"AETHER_OLLAMA_PORT": str(port)}
            output = subprocess.check_output(
                [executable, "plan-objective", args.id, args.request, "ollama", MODEL],
                cwd=ROOT, env=env, timeout=80,
            )
            return json.loads(output)
        stop = threading.Event()
        readings = []
        sampling_errors = []

        def sample():
            try:
                while not stop.is_set():
                    ram = available_ram()
                    if ram < 900 * MIB:
                        raise RuntimeError("available RAM crossed the minimum")
                    readings.append({"ram_available_bytes": ram, **gpu()})
                    stop.wait(0.2)
            except (OSError, RuntimeError, subprocess.SubprocessError) as error:
                sampling_errors.append(error)
                stop_tree(daemon)

        idle_0, cpu_0 = cpu_counts()
        sampler = threading.Thread(target=sample)
        sampler.start()
        try:
            trials = []
            initial = gpu()
            for context in (2048, 4096):
                query(port, "/api/generate", {"model": MODEL, "keep_alive": 0})
                for kind in ("cold", "warmup", "warm", "warm", "warm"):
                    trials.append(measured_trial(port, context, kind))
        finally:
            stop.set()
            sampler.join(timeout=5)
        if sampling_errors:
            raise RuntimeError("resource sampling stopped the benchmark") from sampling_errors[0]
        idle_1, cpu_1 = cpu_counts()
        return dict(scope="one Windows host, one locally installed quantization",
                    model=model, initial_gpu=initial, trials=trials,
                    sampled_gpu_peak_used_mib=max(r["used_mib"] for r in readings),
                    sampled_host_min_free_bytes=min(r["ram_available_bytes"] for r in readings),
                    host_cpu_busy_percent=100 * (1 - (idle_1 - idle_0) / (cpu_1 - cpu_0)),
                    resource_samples=readings)


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("command", choices=("preflight", "benchmark", "draft"))
    cli.add_argument("id", nargs="?")
    cli.add_argument("request", nargs="?")
    args = cli.parse_args()
    if args.command == "draft" and not (args.id and args.request):
        cli.error("draft requires a campaign ID and request JSON")
    print(json.dumps(execute(args), indent=2))
