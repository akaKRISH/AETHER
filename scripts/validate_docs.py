"""Check local AETHER documentation and design assets."""
import argparse
from collections import Counter
from copy import deepcopy
from functools import cache
from hashlib import sha256
import json
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote, urlsplit
from xml.etree import ElementTree

from jsonschema import Draft202012Validator, FormatChecker
from jsonschema.validators import validator_for
from markdown_it import MarkdownIt
import yaml

ROOT = Path(__file__).resolve().parents[1]
MARKDOWN = MarkdownIt("commonmark").enable("table")
RESULTS = Counter()
ERRORS = []


def expect(condition, message):
    if not condition:
        ERRORS.append(message)


def read(path):
    return (ROOT / path).read_text(encoding="utf-8-sig")


def unique_keys(items):
    keys = {}
    for key, value in items:
        if key in keys:
            raise ValueError(f"Duplicate JSON property {key}")
        keys[key] = value
    return keys


def forbid_constant(value):
    raise ValueError(f"Invalid JSON constant {value}")


def parse_json(path):
    return json.loads(read(path), object_pairs_hook=unique_keys, parse_constant=forbid_constant)


@cache
def markdown(path):
    return MARKDOWN.parse(read(path))


def anchors(path):
    result = Counter()
    for i, token in enumerate(markdown(path)):
        if token.type == "heading_open":
            heading = markdown(path)[i + 1]
            plain = "".join(t.content for t in heading.children
                            if t.type in ("text", "code_inline", "image"))
            slug = re.sub(r"[^\w -]", "", plain.lower()).replace(" ", "-")
            result[slug] += 1
            if result[slug] > 1:
                result[f"{slug}-{result[slug] - 1}"] += 1
    result.update(re.findall(r'\bid=["\x27]([^"\x27]+)', read(path)))
    return result


def verify_links(path, candidates):
    for token in markdown(path):
        for child in token.children or ():
            href = child.attrGet("href" if child.type == "link_open" else "src")
            if not href:
                continue
            link = urlsplit(href)
            if link.scheme or link.netloc:
                continue
            RESULTS["relative Markdown destinations"] += 1
            target = (ROOT / path).parent / unquote(link.path) if link.path else ROOT / path
            target = target.resolve()
            if not target.is_relative_to(ROOT):
                expect(False, f"{path}: link leaves repository: {href}")
                continue
            relative = target.relative_to(ROOT).as_posix()
            in_repo = relative in candidates or any(p.startswith(relative + "/") for p in candidates)
            expect(in_repo, f"{path}: missing repository destination {href}")
            if in_repo and link.fragment and target.suffix.lower() == ".md":
                expect(unquote(link.fragment) in anchors(relative), f"{path}: missing heading {href}")


def verify_contracts(parsed):
    formats = FormatChecker()
    expect("date-time" in formats.checkers, "Install jsonschema[format] to check event timestamps")
    schemas = {p: Draft202012Validator(data, format_checker=formats)
               for p, data in parsed.items() if p.endswith(".schema.json")}
    for path, validator in schemas.items():
        try:
            validator.check_schema(validator.schema)
            RESULTS["JSON Schema metaschema passes"] += 1
        except Exception as error:
            expect(False, f"{path}: {error}")
    examples = {p for p in parsed if p.startswith("contracts/examples/")}
    expected_examples = set()
    for path, validator in schemas.items():
        if not path.startswith("contracts/schemas/"):
            continue
        stem = Path(path).name.removesuffix(".schema.json")
        example_path = f"contracts/examples/{stem}.example.json"
        expected_examples.add(example_path)
        if example_path not in parsed:
            continue
        try:
            validator.validate(parsed[example_path])
            RESULTS["working examples accepted"] += 1
        except Exception as error:
            expect(False, f"{example_path}: {error}")
        field, value = {
            "claim": ("verdict", "VERIFIED"),
            "event": ("timestamp", "not-a-timestamp"),
            "protocol": ("network_allowed", True),
            "worker-job": ("input_sha256", "not-a-digest"),
            "worker-result": ("schema_version", "unrecognized"),
        }[stem]
        invalid = deepcopy(parsed[example_path])
        invalid[field] = value
        expect(not validator.is_valid(invalid), f"{path}: accepted negative case {field}")
        RESULTS["working negative cases"] += 1
    expect(expected_examples == examples, "Working schema/example pairing differs")


def verify_archive(files):
    base = "docs/archive/"
    prefix = tuple(base + group + "/" for group in ("v0.1", "v0.2", "genesis", "handoff"))
    expected = {p for p in files if p.startswith(prefix)}
    checked = set()
    for line in read(base + "SHA256SUMS").splitlines():
        digest, name = line.split("  ", 1)
        path = base + name
        expect(path in expected and path not in checked, f"Invalid or repeated archive entry: {name}")
        if path not in expected:
            continue
        checked.add(path)
        expect(sha256((ROOT / path).read_bytes()).hexdigest() == digest, f"Archive hash differs: {name}")
    expect(checked == expected, "Archive inventory is incomplete")
    RESULTS["source files fingerprinted"] = len(checked)


def verify_traceability():
    test_plan = read("docs/testing/TEST_PLAN.md")
    ids = set(re.findall(r"^\| (T\d{2}) \|", test_plan, flags=re.M))
    matrix = read("docs/testing/REQUIREMENTS_TRACEABILITY.md")
    rows = re.findall(r"^\| ((?:PRD-F|TRD-N|SEC-|M1-)\d\d) \| ([^|]+) \|", matrix, flags=re.M)
    defined = set()
    for source, pattern in [
        ("docs/product/FUNCTIONAL_REQUIREMENTS.md", "PRD-F"),
        ("docs/archive/v0.2/AETHER/04_TRD_SRS.md", "TRD-N"),
        ("docs/security/THREAT_MODEL.md", "SEC-"),
        ("docs/roadmap/MILESTONE_01_NATIVE_FIXTURE.md", "M1-"),
    ]:
        defined.update(re.findall(r"^\| (" + pattern + r"\d\d) \|", read(source), flags=re.M))
    mapped = set(req for req, _ in rows)
    expect(mapped == defined and len(mapped) == len(rows), "Traceability is incomplete or repeats IDs")
    for req, tests in rows:
        references = set(re.findall(r"\bT\d{2}\b", tests))
        expect(references and references <= ids, f"{req}: unknown or missing tests")
    RESULTS["requirements mapped"] = len(mapped)
    RESULTS["planned test IDs"] = len(ids)


def verify_diagrams(files):
    sources = {Path(p).stem for p in files if p.startswith("docs/diagrams/") and p.endswith(".mmd")}
    rendered = {Path(p).stem: p for p in files if p.startswith("docs/diagrams/rendered/") and p.endswith(".svg")}
    expect(sources == rendered.keys(), "Editable diagrams and SVG renders must correspond")
    for path in rendered.values():
        element = ElementTree.parse(ROOT / path).getroot()
        expect(element.tag == "{http://www.w3.org/2000/svg}svg" and element.get("viewBox"),
               f"Invalid SVG root or view box: {path}")
        RESULTS["SVG views checked"] += 1


def main():
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("--cff-schema", type=Path, help="Optional local copy of the CFF 1.2.0 schema")
    options = args.parse_args()
    git_files = subprocess.check_output(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=ROOT
    )
    files = {p for p in git_files.decode("utf-8").split("\x00") if p}
    parsed = {}
    for path in sorted(files):
        try:
            if path.endswith(".md"):
                verify_links(path, files)
                RESULTS["Markdown files checked"] += 1
            elif path.endswith(".json"):
                parsed[path] = parse_json(path)
                RESULTS["JSON files parsed"] += 1
        except Exception as error:
            expect(False, f"{path}: {error}")
    cff = yaml.safe_load(read("CITATION.cff"))
    expect(cff.get("cff-version") == "1.2.0" and cff.get("authors"), "Incomplete citation metadata")
    if options.cff_schema:
        schema = json.loads(options.cff_schema.read_text(encoding="utf-8"))
        validator_for(schema)(schema, format_checker=FormatChecker()).validate(cff)
        RESULTS["official CFF instances validated"] += 1
    verify_contracts(parsed)
    verify_archive(files)
    verify_traceability()
    verify_diagrams(files)
    for label, count in sorted(RESULTS.items()):
        print(f"{label}: {count}")
    for error in ERRORS:
        print(f"FAIL {error}", file=sys.stderr)
    return int(bool(ERRORS))


if __name__ == "__main__":
    sys.exit(main())
