#!/usr/bin/env python3
"""Retained, opt-in SAME-HOST Lattice -> existing Aspen CLI/iroh fixture.

Run with `nix develop -c python3 scripts/qualify-render.py --retain NEW_ABSOLUTE_DIR
--lattice ABSOLUTE_LATTICE_BINARY`. The producer build uses its own pinned Nix
shell. Nothing runs at import, in ordinary tests, or from the default graph.
Private node keys/databases stay under .private; no deletion/publication occurs.
"""
import argparse
import json
import os
from pathlib import Path
import platform
import shlex
import shutil
import signal
import socket
import subprocess
import tarfile
import time
import urllib.request

REV = "afc8308bff9a8c50b2315cb4a17c9454252d7c45"
ARCHIVE_ID = "a6307998de7d69f737e1ce39465c63f6742aa4612979321597a304b89771363d"
URL = f"https://codeload.github.com/brittonr/aspen/tar.gz/{REV}"


def dump(path, value):
    with Path(path).open("x") as f:
        json.dump(value, f, indent=2)
        f.write("\n")


def digest(path):
    return subprocess.check_output(["b3sum", "--no-names", str(path)], text=True).strip()


def port():
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Campaign:
    def __init__(self, args):
        self.args = args
        self.repo = Path.cwd().resolve()
        self.root = args.retain.resolve()
        self.root.mkdir(mode=0o700)
        self.private = self.root / ".private"
        self.private.mkdir(mode=0o700)
        self.logs = self.root / "commands"
        self.logs.mkdir()
        self.sequence = 0
        self.processes = {}
        self.metrics = []
        self.summary = {"schema": "lattice.render-qualification.v1", "producer_revision": REV,
                        "scope": "same-host separate processes; not two-host or machine-loss",
                        "status": "running", "controls": {}, "measurements": self.metrics,
                        "operation_pack_identity": None,
                        "operation_pack_reason": "selected route is the exact admitted builtin operation, not an operation pack",
                        "unavailable_phase_timings": {"scheduling_millis": None, "tile_transfer_millis": None,
                            "assembly_millis": None, "retry_cost_millis": None,
                            "reason": "producer CLI exposes no separate phase timing/usage telemetry; command wall times are measured separately"},
                        "cross_backend_comparator": None,
                        "cross_backend_comparator_reason": "not qualified; same exact Wasm/runtime/host only"}

    def invoke(self, args, name, check=True, timeout=650, decode_json=True):
        self.sequence += 1
        stem = self.logs / f"{self.sequence:04}-{name}"
        args = list(map(str, args))
        dump(stem.with_suffix(".invocation.json"), {"argv": args, "cwd": str(self.repo)})
        start = time.monotonic_ns()
        with stem.with_suffix(".stdout").open("xb") as stdout, stem.with_suffix(".stderr").open("xb") as stderr:
            result = subprocess.run(args, stdout=stdout, stderr=stderr, timeout=timeout, cwd=self.repo)
        self.metrics.append({"command": stem.name, "wall_millis": (time.monotonic_ns() - start) // 1_000_000,
                             "exit_code": result.returncode, "remote_cpu_millis": None,
                             "remote_fuel": None, "remote_peak_memory_bytes": None,
                             "missing_usage_reason": "producer result contract supplies no measured usage"})
        if check and result.returncode:
            raise RuntimeError(f"{name} exited {result.returncode}; retained {stem}.stderr")
        data = stem.with_suffix(".stdout").read_bytes()
        if not data.strip() or not decode_json:
            return None
        return json.loads(data)

    def molten(self, command, *args, check=True):
        return self.invoke([self.bin, command, *args], command, check=check)

    def client(self, command, *args, check=True):
        return self.molten(command, "--root", self.client_root, "--target", self.target, *args, check=check)

    def retain_sources(self):
        archive = self.root / "producer.tar.gz"
        urllib.request.urlretrieve(URL, archive)
        if digest(archive) != ARCHIVE_ID:
            raise RuntimeError("anonymous producer acquisition identity mismatch")
        with tarfile.open(archive) as tar:
            for member in tar.getmembers():
                if not member.name.startswith(f"aspen-{REV}") or ".." in Path(member.name).parts:
                    raise RuntimeError("unsafe producer archive member")
            tar.extractall(self.private / "producer", filter="data")
        self.source = self.private / "producer" / f"aspen-{REV}"
        files = subprocess.check_output(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=self.repo).split(b"\0")
        lattice_archive = self.root / "lattice-source.tar.gz"
        with tarfile.open(lattice_archive, "w:gz") as tar:
            for name in sorted(set(files)):
                if name and (self.repo / os.fsdecode(name)).is_file():
                    tar.add(self.repo / os.fsdecode(name), arcname=os.fsdecode(name), recursive=False)
        for lock in ("Cargo.lock", "flake.lock"):
            shutil.copyfile(self.repo / lock, self.root / ("lattice-" + lock))
        self.summary["identities"] = {"producer_archive_blake3": digest(archive),
            "lattice_source_archive_blake3": digest(lattice_archive),
            "lattice_binary_blake3": digest(self.args.lattice),
            "lattice_cargo_lock_blake3": digest(self.repo / "Cargo.lock"),
            "lattice_flake_lock_blake3": digest(self.repo / "flake.lock"),
            "producer_cargo_lock_blake3": digest(self.source / "pilots/orchestration/Cargo.lock"),
            "producer_flake_lock_blake3": digest(self.source / "flake.lock")}
        dump(self.root / "environment.json", {"platform": platform.platform(), "machine": platform.machine(),
            "python": platform.python_version(), "nix": subprocess.check_output(["nix", "--version"], text=True).strip(),
            "producer_url": URL, "build_claim": "provided Lattice binary identity retained, source/build derivation correspondence requires caller build provenance"})

    def build_producer(self):
        target = self.private / "producer-target"
        command = f"CARGO_TARGET_DIR={shlex.quote(str(target))} bash {shlex.quote(str(self.source / 'pilots/orchestration/scripts/render-build.sh'))}"
        # Build output is not JSON; retain independently without invoke decoder.
        build = self.logs / "producer-build.log"
        start = time.monotonic_ns()
        with build.open("xb") as f:
            subprocess.run(["nix-shell", "--pure", str(self.source / "pilots/orchestration/render/shell.nix"),
                            "--run", command], check=True, stdout=f, stderr=subprocess.STDOUT, timeout=3600)
        self.metrics.append({"command": "producer-build", "wall_millis": (time.monotonic_ns() - start) // 1_000_000})
        self.bin = target / "release/molten-orchestrator"
        self.producer_executor_identity = self.invoke([self.args.lattice, "render", "executor-identity", "--file", self.bin], "producer-executor-identity")["identity"]
        self.fixture = target / "release/molten-render-fixture"
        self.component = target / "guests/render.component.wasm"
        shutil.copyfile(self.component, self.root / "render.component.wasm")
        self.summary["identities"].update({"producer_binary_blake3": digest(self.bin), "producer_fixture_blake3": digest(self.fixture),
            "component_blake3": digest(self.component), "component_wit_blake3": digest(target / "guests/render.component.wit")})

    def start(self, name):
        log = (self.logs / f"{name}-process-{time.monotonic_ns()}.log").open("xb")
        child = subprocess.Popen([self.bin, "serve", "--root", self.private / name], stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        log.close()
        self.processes[name] = child

    def stop(self, name, require_interruption=True):
        child = self.processes.pop(name)
        # bounded-exec creates nested process groups, so the owned session is
        # the fixture boundary; killing only the leader's group leaves writers.
        result = subprocess.run(["pkill", "--signal", "KILL", "--session", str(child.pid)],
                                capture_output=True, text=True)
        if result.returncode not in (0, 1):
            result.check_returncode()
        status = child.wait(timeout=15)
        if require_interruption and (result.returncode != 0 or status != -signal.SIGKILL):
            raise RuntimeError(f"requested {name} SIGKILL boundary was not observed")

    def ready(self):
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            result = self.client("status", check=False)
            if result is not None:
                return
            time.sleep(.2)
        raise RuntimeError("producer controller unavailable")

    def setup(self):
        self.policy = self.molten("new-policy", "--out", self.root / "policy.json", "--allow-pure-components")
        self.controller_port, self.worker_port = port(), port()
        self.controller = self.molten("init", "--root", self.private / "controller", "--role", "controller",
            "--policy", self.root / "policy.json", "--bind", f"127.0.0.1:{self.controller_port}")["id"]
        self.client_root = self.private / "client"
        operator = self.molten("init", "--root", self.client_root, "--role", "client", "--policy", self.root / "policy.json")["id"]
        self.worker = self.molten("init", "--root", self.private / "worker", "--role", "worker", "--policy", self.root / "policy.json",
            "--bind", f"127.0.0.1:{self.worker_port}", "--controller", self.controller)["id"]
        self.molten("allow-operator", "--root", self.private / "controller", "--peer", operator)
        self.molten("add-worker", "--root", self.private / "controller", "--peer", self.worker, "--address", f"127.0.0.1:{self.worker_port}")
        self.target = f"{self.controller}@127.0.0.1:{self.controller_port}"
        self.start("worker"); self.start("controller"); self.ready()
        self.invoke([self.fixture, "prepare", self.component, self.root / "producer-prepared"], "producer-prepare")
        self.manifest = json.loads((self.root / "producer-prepared/manifest.json").read_text())

    def prepare(self, name, method="advance", existing=None, fault=None):
        case = self.root / name
        case.mkdir()
        if existing is None:
            authority = {"execution": "lattice-qualification-" + name, "attempt": "attempt-1", "fence": "fence-1",
                         "budget_identity": "budget-" + name, "current": True}
            authority_file = case / "owner-authority.json"
            dump(authority_file, authority)
            manifest = json.loads(json.dumps(self.manifest))
            attempts = (manifest["columns"] * manifest["rows"] + 1) * manifest["runtime"]["max_attempts"]
            request = {"schema": "lattice.render-binding.v1", "execution": authority["execution"], "attempt": authority["attempt"],
                "fence": authority["fence"], "budget": {"identity": authority["budget_identity"],
                    "deadline_epoch_millis": int(time.time() * 1000) + 600_000, "task_attempts": attempts,
                    "fuel": attempts * manifest["runtime"]["fuel"], "task_wall_millis": attempts * manifest["runtime"]["timeout_ms"]},
                "manifest": manifest, "route": {"executable": str(self.bin), "executable_identity": self.producer_executor_identity,
                    "client_root": str(self.client_root), "controller": self.target, "cluster": self.policy["cluster"],
                    "generation": 1, "policy_ref": self.policy["policy_ref"], "fixture_fault": fault},
                "component_file": str(self.component), "authority_file": str(authority_file), "journal_root": str(case / "journal")}
        else:
            request = existing["request"]
        dump(case / "request.json", request)
        config = case / "config.ncl"
        config.write_text('{ data_dir = ' + json.dumps(str(case / "state")) + ', default_timeout_secs = 600, shell = "sh", }\n')
        info = self.invoke([self.args.lattice, "--config", config, "render", "prepare", "--request", case / "request.json",
            "--out", case / "admitted", "--lattice", self.args.lattice, "--repository", self.repo, "--method", method], "lattice-prepare")
        self.invoke([self.args.lattice, "--config", config, "import", case / "admitted/render.ncl"], "lattice-import", decode_json=False)
        required = self.invoke([self.args.lattice, "--config", config, "authority", "inspect", "artifact-bound-render"], "authority-inspect")
        dump(case / "authority-requirements.json", required)
        # The caller explicitly opted into this isolated fixture. Its generated
        # local operator profile grants only this exact reviewed test graph.
        profile = case / "effect-authority.ncl"
        profile.write_text('let contracts = import ' + json.dumps(str(self.repo / "contracts/standalone-effect-authority.ncl")) + ' in\n'
            'let required = import "authority-requirements.json" in\n{ schema = required.schema, workflow_revision = required.workflow_revision, '
            'machine_identity = required.machine_identity, policy_ref = required.policy_ref, grants = required.grants, '
            'expires_at_unix_ms = ' + str(request["budget"]["deadline_epoch_millis"]) + ', roots = {}, } | contracts.StandaloneEffectAuthorityProfile\n')
        config.write_text('{ data_dir = ' + json.dumps(str(case / "state")) + ', effect_authority = ' + json.dumps(str(profile)) +
            ', default_timeout_secs = 600, shell = "sh", }\n')
        return {"case": case, "config": config, "request": request, "info": info}

    def graph(self, case, check=True):
        result = self.invoke([self.args.lattice, "--config", case["config"], "run", "artifact-bound-render", "--output", "json"],
            "lattice-graph-run", check=check)
        dump(case["case"] / "workflow-run.json", result)
        return result

    def record(self, case):
        root = Path(case["request"]["journal_root"]) / case["info"]["binding"]
        files = sorted(root.glob("[0-9][0-9][0-9][0-9].json"))
        if not files:
            raise RuntimeError("no durable Lattice render intent/observation")
        return json.loads(files[-1].read_text())

    def accepted(self, case):
        root = Path(case["request"]["journal_root"]) / case["info"]["binding"]
        accepted = json.loads((root / "accepted-manifest.json").read_text())
        if accepted["execution"] != case["request"]["execution"] or accepted["attempt"] != case["request"]["attempt"]:
            raise RuntimeError("crossed Lattice accepted manifest")
        return accepted, root

    def wait_active(self, case, require_accepted=False):
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            snapshot = self.client("progress", "--run", case["info"]["run"], check=False)
            if snapshot:
                tasks = list(snapshot["tasks"].values())
                active = any(isinstance(t, dict) and ("Running" in t or "Dispatching" in t) for t in tasks)
                accepted = any(isinstance(t, dict) and "Accepted" in t for t in tasks)
                if active and (accepted or not require_accepted):
                    dump(case["case"] / "before-interruption.json", snapshot)
                    return snapshot
                if snapshot["phase"] in ("Succeeded", "Failed", "Cancelled"):
                    break
            time.sleep(.05)
        raise RuntimeError("requested active interruption boundary was not observed; not qualified")

    def run(self):
        self.retain_sources(); self.build_producer(); self.setup()
        first = self.prepare("complete")
        result = self.graph(first)
        if result["failures"] != 0 or result["completed"] != 1:
            raise RuntimeError("actual admitted Lattice graph did not complete")
        accepted, image_root = self.accepted(first)
        repeat = self.prepare("same-backend-repeat")
        self.graph(repeat)
        repeated, repeat_root = self.accepted(repeat)
        if accepted["image"] != repeated["image"] or accepted["tiles"] != repeated["tiles"]:
            raise RuntimeError("same-backend accepted IDs differ")
        if (image_root / "image.ppm").read_bytes() != (repeat_root / "image.ppm").read_bytes():
            raise RuntimeError("same-backend verified presentation bytes differ")
        self.summary["controls"]["real_admitted_graph_and_repeat"] = {"status": "passed", "comparison": "every accepted tile/final content ID and exact PPM bytes",
            "graph": first["info"], "repeat_graph": repeat["info"], "accepted_manifest": str(image_root / "accepted-manifest.json")}
        for control in self.args.controls:
            if control == "lost-ack":
                case = self.prepare("lost-submit-ack", fault="lost-submit-ack")
                self.graph(case, check=False)
                lost = self.record(case)
                if lost["outcome"] != "unknown":
                    raise RuntimeError("lost submit ACK was not Unknown")
                observed = self.prepare("lost-ack-observer", method="observe", existing=case)
                self.graph(observed)
                final = self.record(case)
                if final["outcome"] != "completed":
                    raise RuntimeError("lost ACK did not reconcile by exact accepted identity")
                self.summary["controls"][control] = {"status": "passed", "initial": lost, "recovered": final,
                    "boundary": "real Submit completed; consumer deliberately discarded its ACK; next invocation was Observe, never Submit"}
            elif control in ("controller", "worker"):
                case = self.prepare(control + "-interruption")
                stdout = (case["case"] / "interrupted-graph.stdout").open("xb")
                stderr = (case["case"] / "interrupted-graph.stderr").open("xb")
                child = subprocess.Popen([self.args.lattice, "--config", case["config"], "run", "artifact-bound-render", "--output", "json"], stdout=stdout, stderr=stderr, start_new_session=True)
                self.processes["lattice-observer"] = child
                before = self.wait_active(case, require_accepted=(control == "worker"))
                self.stop(control); self.start(control)
                if control == "worker":
                    self.stop("controller"); self.start("controller")
                self.ready()
                child.wait(timeout=650)
                self.processes.pop("lattice-observer", None); stdout.close(); stderr.close()
                observed = self.prepare(control + "-reobserver", method="observe", existing=case)
                self.graph(observed)
                after = self.client("progress", "--run", case["info"]["run"])
                self.accepted(case)
                for name, task in before["tasks"].items():
                    if isinstance(task, dict) and "Accepted" in task and after["tasks"][name] != task:
                        raise RuntimeError("interruption changed already accepted output")
                self.summary["controls"][control] = {"status": "passed", "before": before, "after": after,
                    "boundary": "actual same-host process SIGKILL/restart; no machine-loss durability claim"}
            elif control == "cancel":
                case = self.prepare("active-cancel")
                stdout = (case["case"] / "cancel-graph.stdout").open("xb")
                stderr = (case["case"] / "cancel-graph.stderr").open("xb")
                child = subprocess.Popen([self.args.lattice, "--config", case["config"], "run", "artifact-bound-render", "--output", "json"], stdout=stdout, stderr=stderr, start_new_session=True)
                self.processes["lattice-observer"] = child
                self.wait_active(case)
                # Disconnect/kill only the local observer first; it cannot prove remote stop.
                self.stop("lattice-observer"); stdout.close(); stderr.close()
                cancel = self.prepare("cancel-observer", method="cancel", existing=case)
                self.graph(cancel, check=False)
                stopped = self.record(case)
                if stopped["outcome"] != "stopped":
                    raise RuntimeError("remote stop not observed; cancellation remains uncertain")
                self.summary["controls"][control] = {"status": "passed", "request_evidence": "retained cancellation intent records", "stopped": stopped}
        self.summary["status"] = "passed"

    def finish(self):
        for name in list(self.processes):
            self.stop(name, require_interruption=False)
        dump(self.root / "qualification.json", self.summary)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--retain", required=True, type=Path)
    parser.add_argument("--lattice", required=True, type=Path)
    parser.add_argument("--controls", nargs="*", choices=("lost-ack", "controller", "worker", "cancel"),
                        default=["lost-ack", "controller", "worker", "cancel"])
    args = parser.parse_args()
    if not args.retain.is_absolute() or not args.lattice.is_absolute():
        parser.error("--retain and --lattice must be absolute")
    if args.retain.resolve().is_relative_to(Path.cwd().resolve()):
        parser.error("--retain must be outside the source tree to avoid recursive source retention")
    campaign = Campaign(args)
    try:
        campaign.run()
    except BaseException as error:
        campaign.summary["status"] = "failed"
        campaign.summary["error"] = str(error)
        raise
    finally:
        campaign.finish()


if __name__ == "__main__":
    main()
