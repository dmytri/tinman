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


# ---- 0. tiers, read from RIGGING.md rather than restated here ----------------
def tiers():
    """Every tier tag RIGGING.md declares under ## Tiers, and the default one.

    Restating this list in source is how the tool silently under-selects: a tier
    added to the rigging and not to the tuple files its scenarios under the
    default tier, and the default sweep excludes them by tag.

    The default tier is read from the rigging alongside the rest, because a
    literal naming it is that same copy in a shorter form. The tag an untagged
    scenario falls back to is the rigging's to declare, and a tool carrying its
    own answer keeps reporting the old one after the rigging renames it.
    """
    text = pathlib.Path("RIGGING.md").read_text()
    block = re.search(r"^## Tiers$(.*?)^## ", text, re.M | re.S)
    if not block:
        raise SystemExit("RIGGING.md declares no ## Tiers section")
    declared = re.findall(r"^- ([a-z-]+):\s*(@\w+)\s*$", block.group(1), re.M)
    found = [tag for _, tag in declared]
    default = next((tag for key, tag in declared if key == "default"), None)
    if not found:
        raise SystemExit("RIGGING.md declares no tier")
    if default is None:
        raise SystemExit("RIGGING.md declares no default tier")
    return found, default


TIERS, DEFAULT_TIER = tiers()

if BASE == "--tiers":
    # The tiers this tool recognises, so a scenario can join them against the
    # rigging that declares them. A tier the rigging names and this list misses
    # files its scenarios under the default tier, and the default sweep excludes
    # them by tag, so the run reports a set it never covered.
    for tag in TIERS:
        print(tag)
    raise SystemExit(0)


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
STEP_ATTR = re.compile(
    r'#\[(given|when|then)\s*\(\s*(?:expr\s*=\s*)?"((?:[^"\\]|\\.)*)"', re.S
)


def step_defs(path="tests/cucumber.rs"):
    """Every step definition's pattern and function name.

    Matched over the whole text rather than line by line: the attribute is
    routinely written across several lines, and a per-line scan misses those,
    which degrades the symbol to the tier-sweep fallback.
    """
    text = pathlib.Path(path).read_text()
    src = text.splitlines()
    starts, pos = [], 0
    for line in src:
        starts.append(pos)
        pos += len(line) + 1
    defs = []
    for m in STEP_ATTR.finditer(text):
        i = max(k for k, off in enumerate(starts) if off <= m.start())
        for j in range(i, min(i + 12, len(src))):
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
def changed_paths(base):
    """Every path the diff touches, Rust or not.

    Read before the Rust filter, because a filter applied silently is how this
    tool under-selects: a diff touching only a scantling, a spec, the rigging or
    this file itself printed the same line as a diff touching nothing at all,
    and a role reading that ran nothing and saw no reason to.
    """
    return [f for f in sh("git", "diff", "--name-only", base).split() if f]


def changed(base):
    files = [f for f in changed_paths(base) if f.endswith(".rs")]
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
IMPL_RULE = '{id: impls, language: rust, rule: {kind: impl_item}}'


def planks_above(src, start_line):
    """Planks in the docblock immediately above the item starting at start_line."""
    found, i = [], start_line - 2
    while i >= 0 and (src[i].strip().startswith("///") or src[i].strip().startswith("#[")):
        m = re.search(r'@planks\("((?:[^"\\]|\\.)*)"\)', src[i])
        if m:
            found.append(m.group(1))
        i -= 1
    return found


def type_planks(src):
    """Planks on each type declaration, keyed by type name."""
    out = {}
    for i, line in enumerate(src):
        m = re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum)\s+(\w+)", line)
        if m:
            got = planks_above(src, i + 1)
            if got:
                out[m.group(1)] = got
    return out


def planks_for(path, ranges):
    """Planks for each touched function, inherited outward where it carries none.

    A trait impl's method is a seam whose plank lives on the impl or on the type
    it implements, so resolving per function alone reports an unplanked symbol
    and reinstates the tier sweep this tool exists to replace.
    """
    src = pathlib.Path(path).read_text().splitlines()
    impls = []
    for m in ast(IMPL_RULE, path):
        r = m.get("range", {})
        s, e = r.get("start", {}).get("line"), r.get("end", {}).get("line")
        text = (m.get("text", "") or "").splitlines()[0] if m.get("text") else ""
        target = re.search(r"\bimpl\b.*?\bfor\s+(\w+)|^\s*impl(?:<[^>]*>)?\s+(\w+)", text)
        if s is not None:
            impls.append({"start": s + 1, "end": e + 1,
                          "type": (target.group(1) or target.group(2)) if target else None})
    by_type = type_planks(src)

    out = collections.defaultdict(list)
    for fr in ranges:
        own = planks_above(src, fr["start"])
        if own:
            out[fr["fn"]] = own
            continue
        enclosing = [im for im in impls if im["start"] <= fr["start"] <= im["end"]]
        enclosing.sort(key=lambda im: im["end"] - im["start"])
        for im in enclosing:
            got = planks_above(src, im["start"]) or by_type.get(im["type"] or "", [])
            if got:
                out[fr["fn"]] = got
                break
    return out


def main():
    scns = scenarios()
    defs = step_defs()
    diff = changed(BASE)
    touched_paths = changed_paths(BASE)
    if not touched_paths:
        print("no changes"); return

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
                    all_patterns.add(bypat[fn]); notes.append(f"{path}:{fn} -> step pattern")
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
                    notes.append(f"{path}:{fn} -> helper, {len(callers)} caller(s)")
                    frontier |= callers
                else:
                    unresolved.append(f"{path}:{fn} (helper, no caller found)")

    # Paths the Rust join cannot reach. A spec selects the scenarios it declares;
    # a scantling selects the scenarios naming it; anything else is named as
    # unresolved so the caller sweeps rather than reading silence as coverage.
    for path in touched_paths:
        if path.endswith(".rs"):
            continue
        if path.endswith(".feature"):
            here = {(sc["file"], sc["name"]) for sc in scns if sc["file"] == path}
            selected |= here
            notes.append(f"spec {path} -> {len(here)} scenario(s)")
        elif path.startswith("scantlings/"):
            named = {(sc["file"], sc["name"]) for sc in scns
                     for step in sc["steps"] if path in step}
            if not named:
                # A rule in the conformance set is reached by the directory the
                # scan configuration names, never by a step naming its path, so
                # the scenario attesting the whole set is what covers it. Left
                # unresolved it would report on every rule edit forever, and a
                # warning that always fires is one nobody reads.
                named = {(sc["file"], sc["name"]) for sc in scns
                         for step in sc["steps"]
                         if "verification-conformance" in path
                         and "rule set" in sc["name"]}
            if named:
                selected |= named
                notes.append(f"scantling {path} -> {len(named)} scenario(s)")
            else:
                unresolved.append(f"{path} (scantling named by no step)")
        else:
            unresolved.append(f"{path} (no join reaches this path)")

    hits = pattern_to_scenarios(scns, all_patterns)
    for p, s in hits.items():
        selected |= s
    dead = [p for p in all_patterns if not hits.get(p)]

    print(f"patterns: {len(all_patterns)}   scenarios selected: {len(selected)}")
    tiers = collections.Counter()
    for f, n in sorted(selected):
        tag = next((t for sc in scns if sc["file"] == f and sc["name"] == n for t in sc["tags"]
                    if t in TIERS), DEFAULT_TIER)
        tiers[tag] += 1
        print(f"  {f}:{n}")
    print("by tier:", dict(tiers))
    # Every path that did reach a join, named with what it reached. The set this
    # tool prints is not an account of the diff: a path can select scenarios
    # that carry another path's name, and reading the selected list is then no
    # way to tell a path that was joined from one the tool passed over. Printing
    # what each path resolved to is what lets a reader hold the report against
    # the diff and see nothing went missing.
    if notes:
        print("accounted for:")
        for n in sorted(set(notes)):
            print("  ", n)
    if dead:
        print(f"patterns binding no scenario: {len(dead)}")
    if unresolved:
        print("unresolved (would fall back):")
        for u in sorted(set(unresolved)):
            print("  ", u)


main()
