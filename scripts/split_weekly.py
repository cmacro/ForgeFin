#!/usr/bin/env python3
"""Split health_company sample data into weekly subdirectories.

Week definition: ISO week (Mon-Sun).
Each row's ISO week is determined by its primary datetime column.
"""

import csv
import datetime
import shutil
from pathlib import Path

SRC_DIR = Path("tests/sample_data/health_company")
DST_ROOT = Path("tests/sample_data/health_company")

# (filename, header_lines_to_skip, datetime_column_index)
FILES = [
    ("order_raw.tsv", 1, 11, "%Y-%m-%d %H:%M:%S"),   # 交易时间
    ("pos_raw.tsv", 1, 0, "%Y-%m-%d %H:%M:%S"),      # 交易时间
    ("bank_raw.tsv", 1, 2, "%Y-%m-%d %H:%M:%S"),     # 交易时间
    ("summary_raw.tsv", 2, None, None),              # use date col 0 (YYYY-MM-DD)
]


def week_label(dt: datetime.datetime) -> str:
    y, w, wd = dt.isocalendar()
    # week start (Monday) for human-readable label
    monday = dt - datetime.timedelta(days=wd - 1)
    sunday = monday + datetime.timedelta(days=6)
    return f"{y}-W{w:02d}_{monday.strftime('%m%d')}-{sunday.strftime('%m%d')}"


def split():
    # Clear existing week directories
    for p in DST_ROOT.iterdir():
        if p.is_dir() and p.name.startswith("week_"):
            shutil.rmtree(p)

    manifest = {}

    for fname, header_count, dt_col, dt_fmt in FILES:
        src = SRC_DIR / fname
        rows = src.read_text(encoding="utf-8").splitlines()
        header_lines = rows[:header_count]
        data_lines = rows[header_count:]

        weekly = {}
        for line in data_lines:
            if not line.strip():
                continue
            if dt_col is None:
                # summary: date is first column (YYYY-MM-DD)
                date_str = line.split("\t", 1)[0]
                dt = datetime.datetime.strptime(date_str, "%Y-%m-%d")
            else:
                cols = line.split("\t")
                dt = datetime.datetime.strptime(cols[dt_col], dt_fmt)
            label = week_label(dt)
            weekly.setdefault(label, []).append(line)

        for label, lines in weekly.items():
            week_dir = DST_ROOT / f"week_{label}"
            week_dir.mkdir(parents=True, exist_ok=True)
            out = week_dir / fname
            out.write_text(
                "\n".join(header_lines + lines) + "\n", encoding="utf-8"
            )
            manifest.setdefault(label, {})[fname] = len(lines)

    # Write manifest
    manifest_path = DST_ROOT / "weekly_manifest.tsv"
    files_order = [f[0] for f in FILES]
    with manifest_path.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f, delimiter="\t")
        w.writerow(["week_dir"] + files_order + ["total"])
        for label in sorted(manifest):
            row = manifest[label]
            total = sum(row.values())
            w.writerow([f"week_{label}"] + [row.get(x, 0) for x in files_order] + [total])

    # Summary
    print(f"{'Week':<28} {'order':>6} {'pos':>6} {'bank':>6} {'summary':>8} {'total':>6}")
    print("-" * 70)
    grand = {f: 0 for f in files_order}
    for label in sorted(manifest):
        row = manifest[label]
        for k in grand:
            grand[k] += row.get(k, 0)
        total = sum(row.values())
        print(
            f"week_{label:<22} "
            f"{row.get('order_raw.tsv', 0):>6} "
            f"{row.get('pos_raw.tsv', 0):>6} "
            f"{row.get('bank_raw.tsv', 0):>6} "
            f"{row.get('summary_raw.tsv', 0):>8} "
            f"{total:>6}"
        )
    print("-" * 70)
    print(
        f"{'TOTAL':<28} "
        f"{grand['order_raw.tsv']:>6} "
        f"{grand['pos_raw.tsv']:>6} "
        f"{grand['bank_raw.tsv']:>6} "
        f"{grand['summary_raw.tsv']:>8} "
        f"{sum(grand.values()):>6}"
    )
    print(f"\nManifest: {manifest_path}")


if __name__ == "__main__":
    split()
