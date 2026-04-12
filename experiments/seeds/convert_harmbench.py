#!/usr/bin/env python3
"""Convert a HarmBench behaviors CSV to the JSONL schema red-queen expects.

Usage:
    python3 convert_harmbench.py <input.csv> <output.jsonl>

Schema per line:
    {"id": "...", "behavior": "...", "category": "...", "tags": [...]}

We use HarmBench's SemanticCategory as `category` (cyber, chemical,
misinformation, ...) because that's the analytically useful split for
per-category analysis. FunctionalCategory (standard/contextual/copyright)
and the raw Tags column are stored in `tags`. BehaviorID becomes `id`.
Only rows with a non-empty Behavior field are emitted. ContextString is
intentionally dropped so each seed is a single self-contained prompt.
"""
import csv
import json
import sys
from collections import Counter


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2

    in_path, out_path = sys.argv[1], sys.argv[2]
    cat_counts: Counter[str] = Counter()
    written = 0

    with open(in_path, newline="", encoding="utf-8") as f_in, \
            open(out_path, "w", encoding="utf-8") as f_out:
        reader = csv.DictReader(f_in)
        for row in reader:
            behavior = (row.get("Behavior") or "").strip()
            if not behavior:
                continue
            functional = (row.get("FunctionalCategory") or "").strip()
            semantic = (row.get("SemanticCategory") or "").strip() or None
            raw_tags = (row.get("Tags") or "").strip()
            bid = (row.get("BehaviorID") or "").strip() or f"hb_{written:05d}"

            tags = [t.strip() for t in raw_tags.split(",") if t.strip()]
            if functional:
                tags.append(f"functional:{functional}")

            entry = {
                "id": bid,
                "behavior": behavior,
                "category": semantic,
                "tags": tags,
            }
            f_out.write(json.dumps(entry, ensure_ascii=False) + "\n")
            cat_counts[semantic or "uncategorized"] += 1
            written += 1

    print(f"wrote {written} behaviors to {out_path}")
    print("category breakdown:")
    for cat, n in sorted(cat_counts.items()):
        print(f"  {cat:<40} {n}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
