"""Generate placeholder Rust tests for unported KaTeX spec suites.

This scans `KaTeX/test/katex-spec.js`, filters out the describe blocks we have
already hand-ported to `aliter/src/spec.rs`, and emits ignored `#[test]` stubs
that simply call `todo!()`. The intent is to provide a to-do list inside
`cargo test` with function names ready to be filled in.

Usage (from repo root):
    python aliter/scripts/generate_katex_spec_todos.py > aliter/tests/katex_spec_todo.rs

Re-run after porting more suites to regenerate the remaining list.
"""

from __future__ import annotations

import pathlib
import re
from collections import Counter

ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = ROOT / "KaTeX" / "test" / "katex-spec.js"

# Describe blocks that are already represented in aliter/src/spec.rs
PORTED = {
    "A parser",
    "An ord parser",
    "A bin parser",
    "A rel parser",
    "A mathinner parser",
    "A punct parser",
    "An open parser",
    "A close parser",
    "A \\\\KaTeX parser",
    "A subscript and superscript parser",
    "A subscript and superscript tree-builder",
    "A parser with limit controls",
    "A group parser",
    "A supsub left/right nucleus parser",
    "An over/underline parser",
    "A \\\\begingroup...\\\\endgroup parser",
    "An implicit group parser",
    "A function parser",
    "An over/brace/brack parser",
    "A genfrac builder",
    "A infix builder",
    "A text parser",
    "A phantom parser",
    "A color parser",
    "A kern parser",
    "A rule parser",
    "A text-mode switch parser",
    "An overbrace underbrace parser",
    "A text font parser",
    "A math font parser",
    "A spacing parser",
    "A text color parser",
    "A delimiter sizing parser",
    "A sqrt parser",
    "A binom parser",
    "A frac parser",
    "A left/right parser",
    "A matrix parser",
    "A phantom sizing parser",
    "A rule color parser",
    "An accent parser",
    "A sizing parser",
    "An arrow parser",
    "A href parser",
}


def snakeify(name: str) -> str:
    """Turn a describe title into a Rust-friendly snake_case identifier."""
    # Replace backslashes and quotes with spaces before collapsing
    clean = re.sub(r"[\\\"']", " ", name)
    # Replace non-alnum with underscores, collapse repeats, strip edges.
    clean = re.sub(r"[^a-zA-Z0-9]+", "_", clean).strip("_").lower()
    if not clean:
        clean = "unknown"
    if not clean[0].isalpha():
        clean = f"katex_{clean}"
    return clean


def main() -> None:
    contents = SPEC.read_text(encoding="utf-8").splitlines()
    describes = []
    for lineno, line in enumerate(contents, 1):
        m = re.match(r'describe\("([^"]+)', line)
        if m:
            title = m.group(1)
            if title in PORTED:
                continue
            describes.append((title, lineno))

    seen = Counter()
    stubs = []
    for title, lineno in describes:
        fn = snakeify(title)
        seen[fn] += 1
        if seen[fn] > 1:
            fn = f"{fn}_{seen[fn]}"
        stubs.append(
            (
                title,
                lineno,
                fn,
            )
        )

    out = []
    out.append("//! Auto-generated placeholders for unported katex-spec.js suites.")
    out.append("//! Regenerate via `python aliter/scripts/generate_katex_spec_todos.py`.")
    out.append("")
    out.append("#![allow(clippy::needless_doctest_main)]")
    out.append("")
    out.append("use aliter::*;")
    out.append("")
    for title, lineno, fn in stubs:
        out.append("#[test]")
        out.append("#[ignore]")
        out.append(f"fn {fn}() {{")
        out.append(f"    // TODO: port \"{title}\" from katex-spec.js:{lineno}")
        out.append("    todo!(\"unported katex-spec.js suite\");")
        out.append("}")
        out.append("")

    print("\n".join(out))


if __name__ == "__main__":  # pragma: no cover - simple script
    main()
