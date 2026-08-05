#!/usr/bin/env python3
"""Derive the exact scenario set a diff selects.

seam/step change -> pattern -> scenarios.  Helpers recurse through their
callers until every path terminates in a pattern.  No tier sweep.
"""
import json, re, subprocess, sys, pathlib, collections

REPO = pathlib.Path(".")
BASE = sys.argv[1] if len(sys.argv) > 1 else "HEAD"


def sh(*cmd, cwd=None):
    return subprocess.run(cmd, capture_output=True, text=True, cwd=cwd or ".").stdout


def ast(rule, path):
    out = sh("ast-grep", "scan", "--inline-rules", rule, "--json=compact", path)
    try:
        return json.loads(out) if out.strip() else []
    except json.JSONDecodeError:
        return []


# ---- 1. scenarios and their steps -------------------------------------------
def scenarios():
    out = []
    for f in sorted(pathlib.Path("features").glob("*.feature")):
        name = None
        tags = []
        pending = []
        for line in f.read_text().splitlines():
            s = line.strip()
            if s.startswith("@"):
                pending = s.split()
            elif s.startswith("Scenario"):
                name = s.split(":", 1)[1].strip() if ":" in s else s
                tags = pending
                pending = []
                out.append({"file": str(f), "name": name, "tags": tags, "steps": []})
            elif s.startswith("Rule:"):
                pending = []
            elif out and re.match(r"^(Given|When|Then|And|But) ", s):
                out[-1]["steps"].append(re.sub(r"^\w+ ", "", s))
    return out


# ---- 2. step definitions: pattern + line range -------------------------------
STEP_ATTR = re.compile(r'#\[(given|when|then)\s*\(\s*(?:expr\s*=\s*)?"((?:[^"\\]|\\.)*)"')


def step_defs(path="tests/cucumber.rs"):
    src = pathlib.Path(path).read_text().splitlines()
    defs = []
    for i, line in enumerate(src):
        m = STEP_ATTR.search(line)
        if not m:
            continue
        for j in range(i + 1, min(i + 6, len(src))):
            fn = re.search(r"\b(?:async\s+)?fn\s+(\w+)", src[j])
            if fn:
                defs.append({"pattern": m.group(2), "fn": fn.group(1), "line": j + 1})
                break
    return defs


# ---- 3. cucumber expression -> regex ----------------------------------------
def to_regex(p):
    out, i = "", 0
    while i < len(p):
        if p.startswith("{string}", i):
            out += r'"[^"]*"'; i += 8
        elif p.startswith("{int}", i):
            out += r"-?\d+"; i += 5
        elif p.startswith("{float}", i):
            out += r"-?[\d.]+"; i += 7
        elif p.startswith("{word}", i):
            out += r"\S+"; i += 6
        elif p.startswith("{}", i):
            out += r".*?"; i += 2
        else:
            out += re.escape(p[i]); i += 1
    return re.compile("^" + out + "$")


def pattern_to_scenarios(scns, patterns):
    rx = {p: to_regex(p) for p in patterns}
    hit = collections.defaultdict(set)
    for sc in scns:
        for step in sc["steps"]:
            for p, r in rx.items():
                if r.match(step):
                    hit[p].add((sc["file"], sc["name"]))
    return hit


# ---- 4. rust function ranges -------------------------------------------------
FN_RULE = '{id: fns, language: rust, rule: {kind: function_item}}'


def fn_ranges(path):
    out = []
    for m in ast(FN_RULE, path):
        r = m.get("range", {})
        s, e = r.get("start", {}).get("line"), r.get("end", {}).get("line")
        name = re.search(r"\bfn\s+(\w+)", m.get("text", "") or "")
        if s is not None and name:
            out.append({"fn": name.group(1), "start": s + 1, "end": e + 1, "file": path})
    return out


# ---- 5. changed lines --------------------------------------------------------
def changed(base):
    files = [f for f in sh("git", "diff", "--name-only", base).split() if f.endswith(".rs")]
    result = {}
    for f in files:
        lines = set()
        cur = None
        for ln in sh("git", "diff", "-U0", base, "--", f).splitlines():
            h = re.match(r"^@@ -\S+ \+(\d+)(?:,(\d+))?", ln)
            if h:
                start = int(h.group(1)); n = int(h.group(2) or 1)
                lines.update(range(start, start + max(n, 1)))
        result[f] = lines
    return result


# ---- 6. planks on production seams ------------------------------------------
def planks_for(path, ranges):
    src = pathlib.Path(path).read_text().splitlines()
    out = collections.defaultdict(list)
    for fr in ranges:
        i = fr["start"] - 2
        while i >= 0 and (src[i].strip().startswith("///") or src[i].strip().startswith("#[")):
            m = re.search(r'@planks\("((?:[^"\\]|\\.)*)"\)', src[i])
            if m:
                out[fr["fn"]].append(m.group(1))
            i -= 1
    return out


def main():
    scns = scenarios()
    defs = step_defs()
    diff = changed(BASE)
    if not diff:
        print("no rust changes"); return

    selected, unresolved, notes = set(), [], []
    all_patterns = set()

    for path, lines in diff.items():
        if not lines:
            continue
        ranges = [r for r in fn_ranges(path) if lines & set(range(r["start"], r["end"] + 1))]
        touched = {r["fn"] for r in ranges}

        if path.startswith("src/"):
            pl = planks_for(path, ranges)
            for fn in touched:
                if pl.get(fn):
                    all_patterns.update(pl[fn]); notes.append(f"seam {path}:{fn} -> {len(pl[fn])} plank(s)")
                else:
                    unresolved.append(f"{path}:{fn} (no plank)")
        else:
            bypat = {d["fn"]: d["pattern"] for d in defs}
            frontier, seen = set(touched), set()
            while frontier:
                fn = frontier.pop()
                if fn in seen:
                    continue
                seen.add(fn)
                if fn in bypat:
                    all_patterns.add(bypat[fn]); notes.append(f"step {fn} -> pattern")
                    continue
                # helper: find its callers inside tests/
                callers = set()
                for tf in pathlib.Path("tests").rglob("*.rs"):
                    txt = tf.read_text()
                    if re.search(r"\b" + re.escape(fn) + r"\s*\(", txt):
                        for r in fn_ranges(str(tf)):
                            body = "\n".join(txt.splitlines()[r["start"] - 1:r["end"]])
                            if re.search(r"\b" + re.escape(fn) + r"\s*\(", body) and r["fn"] != fn:
                                callers.add(r["fn"])
                if callers:
                    notes.append(f"helper {fn} -> {len(callers)} caller(s)")
                    frontier |= callers
                else:
                    unresolved.append(f"{path}:{fn} (helper, no caller found)")

    hits = pattern_to_scenarios(scns, all_patterns)
    for p, s in hits.items():
        selected |= s
    dead = [p for p in all_patterns if not hits.get(p)]

    print(f"patterns: {len(all_patterns)}   scenarios selected: {len(selected)}")
    tiers = collections.Counter()
    for f, n in sorted(selected):
        tag = next((t for sc in scns if sc["file"] == f and sc["name"] == n for t in sc["tags"]
                    if t in ("@sandbox", "@inference", "@advisory")), "@logic")
        tiers[tag] += 1
        print(f"  {f}:{n}")
    print("by tier:", dict(tiers))
    if dead:
        print(f"patterns binding no scenario: {len(dead)}")
    if unresolved:
        print("unresolved (would fall back):")
        for u in sorted(set(unresolved)):
            print("  ", u)


main()
