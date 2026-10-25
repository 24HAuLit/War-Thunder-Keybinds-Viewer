#!/usr/bin/env python3
"""Reproduce the bundled BLK presets with Python 3.9+ and no third-party packages."""

import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import sys
import unittest
import urllib.request


REPOSITORY = "gszabi99/War-Thunder-Datamine"
COMMIT = "aa326f10c6ba7d523647572368ead2e4d51ffb4d"
COMMIT_DATE = "2026-10-23T13:44:25Z"
VERSION = "2.59.0.86"
UPSTREAM_DIRECTORY = "aces.vromfs.bin_u/config/hotkeys"
DATA_DIRECTORY = Path(__file__).resolve().parents[1] / "src/bindings/presets/data"
ROOT_PRESETS = (
    "hotkey.keyboard_ver1.blk",
    "hotkey.base.keyboard_simulator.blk",
    "hotkey.base.keyboard.blk",
    "hotkey.empty_ver1.blk",
)
# Exact source-byte hashes also form the download allowlist. Never crawl a tree
# or accept arbitrary paths/URLs supplied by a preset.
SOURCE_SHA256 = {
    "hotkey.keyboard_ver1.blk": "f36e042a5bac84281149bdae3876635fc3a9fd7429db7aebda390a85eaf516e9",
    "hotkey.base.keyboard_simulator.blk": "54edd562e6491b094c27f582cdfe8f4eaee2adc309eaea69e011c4873a19147b",
    "hotkey.base.keyboard.blk": "9174dbe9084d22a18cb21500f76d8037f8fd7850842ecf23ca9262506dc3aa1c",
    "hotkey.empty_ver1.blk": "a378796fef5873e9e48dd919534d0f1c43730ddd6a03afeee26bcb7e0dc2be1e",
    "hotkey.event.human.blk": "e3d7b37ef1684302fece926110da011b92b5d1bc916c80f864b1f6f8dd33b55b",
}
SAFE_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_.]*\Z")


def sha256(content):
    return hashlib.sha256(content).hexdigest()


def source_url(name):
    if name not in SOURCE_SHA256:
        raise ValueError(f"preset is not allowlisted: {name!r}")
    return (
        f"https://raw.githubusercontent.com/{REPOSITORY}/{COMMIT}/"
        f"{UPSTREAM_DIRECTORY}/{name}x"
    )


def quote(value):
    """Dagor escapes, not JSON/backslash escapes; escape tildes first."""
    if any(ord(char) < 32 and char not in "\n\r\t" for char in value):
        raise ValueError("unsupported control character in BLK string")
    return '"' + (
        value.replace("~", "~~")
        .replace('"', '~"')
        .replace("\n", "~n")
        .replace("\r", "~r")
        .replace("\t", "~t")
    ) + '"'


def scalar(value):
    # bool is a subclass of int in Python.
    if isinstance(value, bool):
        return "b", "yes" if value else "no"
    if isinstance(value, int):
        if not -(2**31) <= value < 2**31:
            raise ValueError(f"integer does not fit the parser's i32 type: {value}")
        return "i", str(value)
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ValueError("non-finite BLK real")
        return "r", repr(value)
    if isinstance(value, str):
        return "t", quote(value)
    raise ValueError(f"unsupported JSON value: {value!r}")


def convert(document):
    """Keep object order and empty objects; arrays repeat the same entry name."""
    if not isinstance(document, dict):
        raise ValueError("the preset root must be an object")
    lines = []

    def emit(name, value, depth):
        label = name if SAFE_NAME.fullmatch(name) else quote(name)
        indent = "  " * depth
        if isinstance(value, dict):
            if depth >= 128:
                raise ValueError("preset exceeds the existing parser's nesting limit")
            lines.append(f"{indent}{label} {{")
            for child_name, child in value.items():
                emit(child_name, child, depth + 1)
            lines.append(f"{indent}}}")
        elif isinstance(value, list):
            # BLK has no untyped empty-array representation. Do not silently
            # drop something a future source might introduce.
            if not value:
                raise ValueError(f"unrepresentable empty array at {name!r}")
            if any(isinstance(item, list) for item in value):
                raise ValueError(f"unsupported nested array at {name!r}")
            for item in value:
                emit(name, item, depth)
        else:
            kind, text = scalar(value)
            lines.append(f"{indent}{label}:{kind}={text}")

    for name, value in document.items():
        emit(name, value, 0)
    return ("\n".join(lines) + "\n").encode("utf-8")


def unique_object(pairs):
    result = {}
    for name, value in pairs:
        if name in result:
            raise ValueError(f"duplicate JSON object key: {name!r}")
        result[name] = value
    return result


def decode(content):
    document = json.loads(content.decode("utf-8"), object_pairs_hook=unique_object)
    if not isinstance(document, dict) or not isinstance(document.get("controls"), dict):
        raise ValueError("preset must contain a controls object")
    return document


def default_dependencies(document):
    paths = document["controls"].get("basePresetPaths", {})
    if not isinstance(paths, dict):
        raise ValueError("basePresetPaths must be an object")
    defaults = paths.get("default", [])
    if not isinstance(defaults, list):
        defaults = [defaults]
    dependencies = []
    for path in defaults:
        if not isinstance(path, str) or not path.startswith("config/hotkeys/"):
            raise ValueError(f"unexpected default base path: {path!r}")
        name = path.removeprefix("config/hotkeys/")
        if name not in SOURCE_SHA256:
            raise ValueError(f"default base is not allowlisted: {path!r}")
        dependencies.append(name)
    return dependencies


def generate(fetch=False, check=False):
    sources = {}
    documents = {}
    dependencies = {}
    visiting = set()

    def visit(name):
        if name in visiting:
            raise ValueError(f"default base dependency cycle at {name}")
        if name in documents:
            return
        visiting.add(name)
        path = DATA_DIRECTORY / "source" / f"{name}x"
        if fetch:
            request = urllib.request.Request(
                source_url(name), headers={"User-Agent": "wt-keybind-viewer-presets"}
            )
            with urllib.request.urlopen(request, timeout=60) as response:
                content = response.read()
        else:
            content = path.read_bytes()
        if sha256(content) != SOURCE_SHA256[name]:
            raise ValueError(f"source checksum mismatch: {name}x")
        document = decode(content)
        bases = default_dependencies(document)
        for base in bases:
            visit(base)
        visiting.remove(name)
        sources[name] = content
        documents[name] = document
        dependencies[name] = bases

    for name in ROOT_PRESETS:
        visit(name)
    if set(documents) != set(SOURCE_SHA256):
        raise ValueError("dependency closure differs from the pinned source allowlist")

    # Validate the whole closure before writing any output.
    outputs = {}
    files = []
    for name in sorted(documents):
        content = convert(documents[name])
        outputs[name] = content
        if fetch:
            outputs[f"source/{name}x"] = sources[name]
        files.append({
            "source_path": f"{UPSTREAM_DIRECTORY}/{name}x",
            "source_url": source_url(name),
            "source_file": f"source/{name}x",
            "source_sha256": SOURCE_SHA256[name],
            "generated_file": name,
            "generated_sha256": sha256(content),
            "controls_version": documents[name]["controls"].get("version"),
            "default_dependencies": dependencies[name],
        })
    manifest = {
        "schema_version": 1,
        "repository": f"https://github.com/{REPOSITORY}",
        "commit": COMMIT,
        "commit_date": COMMIT_DATE,
        "commit_version_label": VERSION,
        "revision_resolution": {
            "api_url": f"https://api.github.com/repos/{REPOSITORY}/commits/master",
            "resolved_on": "2026-10-25",
        },
        "license": {
            "spdx_identifier": None,
            "repository_api_license": None,
            "pinned_license_api_status": 404,
            "pinned_license_api_url": (
                f"https://api.github.com/repos/{REPOSITORY}/license?ref={COMMIT}"
            ),
            "notice": (
                "Datamined War Thunder game data; rights remain with their respective "
                "owners (including Gaijin Entertainment). No affirmative redistribution "
                "license was identified. The viewer's license does not relicense this data."
            ),
        },
        "roots": list(ROOT_PRESETS),
        "files": files,
    }
    outputs["manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode("utf-8")
    if check:
        mismatches = [
            relative for relative, content in outputs.items()
            if not (DATA_DIRECTORY / relative).is_file()
            or (DATA_DIRECTORY / relative).read_bytes() != content
        ]
        if mismatches:
            raise ValueError("generated files differ: " + ", ".join(mismatches))
        print(f"Verified {len(files)} presets, source checksums, and manifest (offline).")
    else:
        for relative, content in outputs.items():
            path = DATA_DIRECTORY / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        print(f"Generated {len(files)} presets and manifest at {COMMIT}.")


class ConversionTests(unittest.TestCase):
    def test_order_repetition_and_empty_blocks(self):
        document = {"controls": {
            "version": 5,
            "hotkeys": {"action": [{}, {"keyboardKey": [56, 29]}, {}]},
            "axes": {},
            "params": {"enabled": True, "disabled": False, "gain": 0.5},
        }}
        self.assertEqual(convert(document).decode("utf-8"), (
            "controls {\n"
            "  version:i=5\n"
            "  hotkeys {\n"
            "    action {\n    }\n"
            "    action {\n      keyboardKey:i=56\n      keyboardKey:i=29\n    }\n"
            "    action {\n    }\n"
            "  }\n"
            "  axes {\n  }\n"
            "  params {\n    enabled:b=yes\n    disabled:b=no\n    gain:r=0.5\n  }\n"
            "}\n"
        ))

    def test_dagor_escapes_and_quoted_names(self):
        self.assertEqual(quote('~"\n\r\t'), '"~~~"~n~r~t"')
        self.assertEqual(convert({'odd name': 'é~"\n\r\t'}),
                         '"odd name":t="é~~~"~n~r~t"\n'.encode("utf-8"))
        self.assertEqual(quote("config/hotkeys/base.blk"), '"config/hotkeys/base.blk"')

    def test_scalar_arrays_and_float_type(self):
        self.assertEqual(convert({"x": [True, 1, 1.0, "s", False]}),
                         b'x:b=yes\nx:i=1\nx:r=1.0\nx:t="s"\nx:b=no\n')

    def test_unsupported_values_fail(self):
        for value in (None, [], [[1]], float("nan"), float("inf"), 2**31, "\x00"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                convert({"x": value})

    def test_duplicate_json_keys_fail(self):
        with self.assertRaises(ValueError):
            decode(b'{"controls": {}, "controls": {}}')

    def test_dependencies_are_allowlisted(self):
        document = {"controls": {"basePresetPaths": {
            "default": "config/hotkeys/hotkey.empty_ver1.blk",
        }}}
        self.assertEqual(default_dependencies(document), ["hotkey.empty_ver1.blk"])
        document["controls"]["basePresetPaths"]["default"] = "config/hotkeys/../other.blk"
        with self.assertRaises(ValueError):
            default_dependencies(document)
        self.assertEqual(default_dependencies({"controls": {}}), [])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--fetch", action="store_true",
                       help="redownload only pinned, allowlisted sources; verify hashes and regenerate")
    modes.add_argument("--check", action="store_true",
                       help="verify source hashes and generated bytes without writes or network")
    modes.add_argument("--self-test", action="store_true",
                       help="run converter regression tests without writes or network")
    args = parser.parse_args()
    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(ConversionTests)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1
    try:
        generate(fetch=args.fetch, check=args.check)
    except (OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
