#!/usr/bin/env python3
"""Render a `zanary-probe test --summary` file as a Markdown block for a CI job summary.

    scripts/ci-summary.py runs/private/test.json [--title "private flow"] >> "$GITHUB_STEP_SUMMARY"

It reads only the summary the gate wrote, so a job's summary can never claim more than the gate's
own numbers — and when the gate crashed before writing one, this says that instead of inventing a
table.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

EMOJI = {"as-expected": "✅", "leaked": "❌", "control-not-caught": "❌", "rig-error": "🛠️"}


def shorten(value: str, head: int = 8, tail: int = 4) -> str:
    if not value or len(value) <= head + tail + 1:
        return value or "—"
    return f"{value[:head]}…{value[-tail:]}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("summary", help="the test.json written by `zanary-probe test --summary`")
    parser.add_argument("--title", default=None, help="what this run was, in the caller's words")
    args = parser.parse_args()

    path = Path(args.summary)
    if not path.exists():
        print(f"## ZANARY gate — {args.title or 'run'}\n")
        print(f"No `{path}`: the gate did not get far enough to write a summary. See the step log.")
        return 0

    summary = json.loads(path.read_text())
    verdict = summary.get("verdict", "unknown")
    icon = EMOJI.get(verdict, "•")
    title = args.title or summary.get("flow_id", "flow")

    print(f"## ZANARY gate — {title}\n")
    print(f"**{icon} {verdict}** (exit {summary.get('exit_code')}) — "
          f"expected `{summary.get('expectation')}`, got {summary.get('fail')} failed property/ies\n")

    print("| | |")
    print("|---|---|")
    print(f"| flow | `{summary.get('flow_id')}` @ `{shorten(summary.get('flow_commit',''))}` |")
    print(f"| definition | `{summary.get('flow_definition')}` |")
    print(f"| properties | {summary.get('pass')} pass · **{summary.get('fail')} fail** · "
          f"{summary.get('skip')} skip |")
    if summary.get("failed_properties"):
        print(f"| caught | {', '.join('`' + p + '`' for p in summary['failed_properties'])} |")
    checks = summary.get("chain_checks") or {}
    chain = "✅ VERIFIED" if summary.get("chain_verified") else "❌ NOT VERIFIED"
    print(f"| chain verification | {chain} ({checks.get('pass', 0)} pass · "
          f"{checks.get('fail', 0)} fail · {checks.get('skip', 0)} skip) |")
    anchor = summary.get("anchor")
    if anchor:
        print(f"| anchor | height {anchor.get('height')} · tx `{shorten(anchor.get('txid',''))}` |")
        print(f"| beacon UFVK | `{shorten(anchor.get('beacon_ufvk',''), 16, 8)}` |")
    else:
        print("| anchor | none on this run |")
    print(f"| verified at | height {summary.get('verified_at_height')} · {summary.get('verified_at')} |")
    print(f"| claim expires | {summary.get('claim_expires_at') or 'never (no clock)'} |")
    print(f"| report hash | `{shorten(summary.get('report_hash',''), 16, 8)}` |")
    print(f"| evidence | `{summary.get('run_dir')}` (uploaded as an artifact) |")
    print()
    if anchor:
        print("Re-check this run with no trust in the rig that produced it:")
        print()
        print("```sh")
        print(f"zanary-report verify {summary.get('report_file')} --node <node>")
        print(f"zanary-report verify-anchor --report {summary.get('report_file')} \\")
        print(f"    --beacon-ufvk {anchor.get('beacon_ufvk')} --node <node>")
        print("```")
    return 0


if __name__ == "__main__":
    sys.exit(main())
