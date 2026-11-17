#!/usr/bin/env python3
"""The test that confirms Kavat's model: the cases of cases.jsonl, asked of each candidate
through Ollama, four times, and scored without a judge.

usage: run.py --dry-run                 check the catalogue and every case; no network
       run.py                           every candidate, four runs
       run.py --models a,b --runs 1 --limit 20 --out out/agent-bench/try

How a case is written and scored is in README.md, beside this file.
"""

import argparse
import glob
import json
import os
import re
import statistics
import sys
import time
import urllib.error
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))

# The candidates, in the design's order. The tags are the ones the report gives; it gives none
# for Ministral 3 3B, so that one is filled in here before the first run.
PLACEHOLDER = "FILL-IN"
CANDIDATES = [
    "qwen3.5:4b-q4_K_M",
    PLACEHOLDER + ":ministral-3-3b",
    "qwen3:4b-instruct",
    "lfm3:1.7b-q4_K_M",
]

LEVELS = ["Read", "Reversible", "With snapshot", "Privileged"]
CONFIRMS = ["none", "log", "once", "password"]
KINDS = ["tool", "answer", "none", "refuse"]
STYLES = ["plain", "indirect", "sloppy", "normalise"]
MATCHERS = ["iexact", "one_of", "number", "range", "contains", "regex", "absent_or", "any"]
TYPES = {"string": str, "integer": int, "number": (int, float), "boolean": bool}

# What a reply can be. Only the first is right.
OUTCOMES = ["correct", "wrong_tool", "wrong_args", "invalid_args", "unknown_tool", "missed", "acted",
            "wrong_reply", "empty", "error"]


class Bad(Exception):
    pass


# --- the catalogue and the cases ---------------------------------------------------------------

def load_tools(path):
    with open(path) as f:
        catalogue = json.load(f)
    return {t["name"]: t for t in catalogue["tools"]}, catalogue


def load_cases(path):
    cases = []
    with open(path) as f:
        for number, line in enumerate(f, 1):
            if not line.strip():
                continue
            try:
                cases.append(json.loads(line))
            except ValueError as e:
                raise Bad("%s, line %d: not JSON: %s" % (path, number, e))
    return cases


def check_value(schema, value):
    """Why a value does not fit its parameter, or None."""
    kind = TYPES[schema["type"]]
    if isinstance(value, bool) and schema["type"] != "boolean":
        return "a boolean where %s goes" % schema["type"]
    if not isinstance(value, kind):
        return "%r is not %s" % (value, schema["type"])
    if "enum" in schema and value not in schema["enum"]:
        return "%r is not one of %s" % (value, ", ".join(schema["enum"]))
    if "minimum" in schema and value < schema["minimum"] or "maximum" in schema and value > schema["maximum"]:
        return "%r is out of %s to %s" % (value, schema.get("minimum"), schema.get("maximum"))
    return None


def check_args(tool, args):
    """Why a call's arguments do not fit the tool, or None. It is what kavatd would refuse."""
    if not isinstance(args, dict):
        return "the arguments are not an object"
    props = tool["parameters"]["properties"]
    for name in tool["parameters"]["required"]:
        if name not in args:
            return "%s is missing" % name
    for name, value in args.items():
        if name not in props:
            return "%s is not a parameter" % name
        why = check_value(props[name], value)
        if why:
            return "%s: %s" % (name, why)
    return None


def check_catalogue(tools, catalogue):
    problems = []
    if len(tools) != len(catalogue["tools"]):
        problems.append("tools.json: a name is there twice")
    for name, tool in tools.items():
        where = "tools.json, %s: " % name
        if not re.fullmatch(r"[a-z]+\.[a-z_]+", name):
            problems.append(where + "the name is not area.verb")
        if tool.get("level") not in LEVELS:
            problems.append(where + "level is not one of the design's four")
        if tool.get("confirm") not in CONFIRMS:
            problems.append(where + "confirm is not one of " + ", ".join(CONFIRMS))
        if not tool.get("description") or "\n" in tool["description"]:
            problems.append(where + "the description is not one line")
        params = tool.get("parameters", {})
        if params.get("type") != "object" or not isinstance(params.get("properties"), dict):
            problems.append(where + "parameters is not an object with properties")
            continue
        for required in params.get("required", []):
            if required not in params["properties"]:
                problems.append(where + "%s is required and is not a parameter" % required)
        for prop, schema in params["properties"].items():
            if schema.get("type") not in TYPES:
                problems.append(where + "%s has no type the test knows" % prop)
            elif "enum" in schema and schema["type"] != "string":
                problems.append(where + "%s: an enum that is not of strings" % prop)
            if not schema.get("description"):
                problems.append(where + "%s has no description" % prop)
    return problems


def check_matcher(schema, want):
    """Why what a case expects of an argument could never be met, or None."""
    if not isinstance(want, dict):
        return check_value(schema, want)
    if len(want) != 1 and set(want) != {"number", "tol"}:
        return "a rule has one key"
    rule = next(k for k in want if k != "tol")
    value = want[rule]
    if rule not in MATCHERS:
        return "%s is not a rule" % rule
    if rule in ("iexact", "absent_or"):
        return check_value(schema, value)
    if rule == "one_of":
        if not isinstance(value, list) or not value:
            return "one_of takes a list"
        if "enum" in schema:
            return next((check_value(schema, v) for v in value if check_value(schema, v)), None)
    if rule == "contains" and (schema["type"] != "string" or not isinstance(value, list)):
        return "contains takes a list of words, on a string"
    if rule == "regex":
        try:
            re.compile(value)
        except re.error as e:
            return "the regex does not compile: %s" % e
    if rule == "range" and not (isinstance(value, list) and len(value) == 2 and value[0] <= value[1]):
        return "range takes [lowest, highest]"
    if rule in ("range", "number") and schema["type"] not in ("integer", "number"):
        return "%s on a parameter that is not a number" % rule
    return None


def check_cases(cases, tools):
    problems, seen = [], set()
    for case in cases:
        cid = case.get("id", "?")
        where = "cases.jsonl, %s: " % cid
        if cid in seen:
            problems.append(where + "the id is there twice")
        seen.add(cid)
        for key in ("id", "category", "style", "say", "offer", "expect"):
            if key not in case:
                problems.append(where + "no %s" % key)
        if any(k not in case for k in ("offer", "expect", "say")):
            continue
        if case.get("style") not in STYLES:
            problems.append(where + "style is not one of " + ", ".join(STYLES))
        offer = case["offer"]
        if not 5 <= len(offer) <= 8 or len(set(offer)) != len(offer):
            problems.append(where + "offers %d tools, or one twice; five to eight go" % len(offer))
        for name in offer:
            if name not in tools:
                problems.append(where + "offers %s, which is not in the catalogue" % name)
        expect = case["expect"]
        if expect.get("kind") not in KINDS:
            problems.append(where + "kind is not one of " + ", ".join(KINDS))
            continue
        if expect["kind"] == "tool":
            if not expect.get("calls"):
                problems.append(where + "expects a tool and names none")
            for call in expect.get("calls", []):
                name = call.get("tool")
                if name not in offer:
                    problems.append(where + "expects %s, which is not on offer" % name)
                if name not in tools:
                    continue
                props = tools[name]["parameters"]["properties"]
                for arg, want in call.get("args", {}).items():
                    if arg not in props:
                        problems.append(where + "%s has no parameter %s" % (name, arg))
                        continue
                    why = check_matcher(props[arg], want)
                    if why:
                        problems.append(where + "%s.%s: %s" % (name, arg, why))
        elif "calls" in expect:
            problems.append(where + "expects no tool and names one")
        for message in case.get("messages", []):
            if message.get("role") not in ("assistant", "tool", "user"):
                problems.append(where + "a message with no role the test knows")
            for call in message.get("tool_calls", []):
                name = call["function"]["name"]
                if name not in offer:
                    problems.append(where + "the talk calls %s, which is not on offer" % name)
                elif check_args(tools[name], call["function"]["arguments"]):
                    problems.append(where + "the talk calls %s badly: %s" % (name, check_args(tools[name], call["function"]["arguments"])))
        if case.get("messages") and case["messages"][-1]["role"] == "assistant":
            problems.append(where + "the talk ends with the assistant: there is nothing to answer")
    return problems


# --- scoring -----------------------------------------------------------------------------------

def coerce(tool, args):
    """What kavatd would do before checking: "50" where a number goes is 50, "true" is true,
    and "Nord" is nord where the values are a closed list."""
    props = tool["parameters"]["properties"]
    out = {}
    for name, value in args.items():
        kind = props.get(name, {}).get("type")
        if isinstance(value, str):
            text = value.strip().lower()
            if text in props.get(name, {}).get("enum", []):
                value = text
            elif kind in ("integer", "number") and re.fullmatch(r"[+-]?\d+(\.0+)?%?", text):
                value = int(float(text.rstrip("%")))
            elif kind == "boolean" and text in ("true", "false"):
                value = text == "true"
        elif isinstance(value, float) and kind == "integer" and value == int(value):
            value = int(value)
        out[name] = value
    return out


def norm(value):
    return " ".join(str(value).lower().split())


def matches(want, args, name):
    """Whether an argument is what the case expects. A bare value is matched exactly."""
    present = name in args
    got = args.get(name)
    if not isinstance(want, dict):
        return present and type(got) is type(want) and got == want
    if "absent_or" in want:
        return not present or got == want["absent_or"]
    if not present:
        return False
    if "any" in want:
        return True
    if "iexact" in want:
        return norm(got) == norm(want["iexact"])
    if "one_of" in want:
        return norm(got) in [norm(v) for v in want["one_of"]]
    if "contains" in want:
        return all(norm(word) in norm(got) for word in want["contains"])
    if "regex" in want:
        return re.search(want["regex"], norm(got), re.IGNORECASE) is not None
    if isinstance(got, bool) or not isinstance(got, (int, float)):
        return False
    if "range" in want:
        return want["range"][0] <= got <= want["range"][1]
    if "number" in want:
        return abs(got - want["number"]) <= want.get("tol", 0)
    return False


def looks_like_a_call(text):
    return bool(re.search(r'<tool_call|<function=|"name"\s*:\s*"[a-z]+\.[a-z_]+"|^\s*[a-z]+\.[a-z_]+\(', text))


def score(case, tools, reply):
    """(outcome, why). reply is what ask() returns."""
    expect = case["expect"]
    if reply.get("error"):
        return "error", reply["error"]
    calls, text = reply["calls"], reply["content"].strip()
    if not calls and not text:
        return "empty", "nothing came back, after %d tokens" % reply.get("eval_count", 0)
    if expect["kind"] != "tool":
        if calls:
            return "acted", "called %s where no tool goes" % calls[0]["name"]
        if looks_like_a_call(text):
            return "acted", "wrote a call as text"
        wanted = expect.get("reply_has")
        if wanted and not any(norm(w) in norm(text) for w in wanted):
            return "wrong_reply", "the reply names none of: " + ", ".join(wanted)
        return "correct", ""
    if not calls:
        return "missed", "wrote a call as text" if looks_like_a_call(text) else "answered in words"
    # One step per command: the first call is the one that counts.
    name, args = calls[0]["name"], calls[0]["args"]
    if name not in case["offer"]:
        return "unknown_tool", "%s is not on offer" % name
    wanted = [c for c in expect["calls"] if c["tool"] == name]
    if not wanted:
        return "wrong_tool", "%s, not %s" % (name, " or ".join(c["tool"] for c in expect["calls"]))
    args = coerce(tools[name], args if isinstance(args, dict) else {})
    why = check_args(tools[name], args)
    if why:
        return "invalid_args", why
    for call in wanted:
        off = [arg for arg, want in call.get("args", {}).items() if not matches(want, args, arg)]
        if not off:
            return "correct", ""
    return "wrong_args", "%s: %s" % (", ".join(off), json.dumps({a: args.get(a) for a in off}))


# --- Ollama ------------------------------------------------------------------------------------

class Ollama:
    def __init__(self, host):
        self.host = host.rstrip("/")
        self.no_think = set()

    def call(self, path, body=None, timeout=30):
        data = json.dumps(body).encode() if body is not None else None
        request = urllib.request.Request(self.host + path, data=data, headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(request, timeout=timeout) as answer:
            return json.load(answer)

    def version(self):
        try:
            return self.call("/api/version")["version"]
        except (OSError, ValueError, KeyError) as e:
            raise Bad("Ollama does not answer at %s (%s).\nStart it with: systemctl start ollama, or: ollama serve" % (self.host, e))

    def pulled(self):
        return {m["name"] for m in self.call("/api/tags")["models"]}

    def load(self, model, num_ctx):
        """Load a model and keep it; how long the load took."""
        start = time.monotonic()
        self.call("/api/generate", {"model": model, "keep_alive": "30m", "options": {"num_ctx": num_ctx}}, timeout=600)
        return time.monotonic() - start

    def unload(self, model):
        try:
            self.call("/api/generate", {"model": model, "keep_alive": 0}, timeout=60)
        except OSError:
            pass

    def memory(self, model):
        for entry in self.call("/api/ps").get("models", []):
            if entry.get("name") == model or entry.get("model") == model:
                return {"size": entry.get("size", 0), "size_vram": entry.get("size_vram", 0)}
        return {"size": 0, "size_vram": 0}

    def ask(self, model, messages, offered, options, timeout):
        """One request, streamed. Calls, text, and how long the first piece and the whole took."""
        body = {"model": model, "messages": messages, "tools": offered, "stream": True, "keep_alive": "30m",
                "options": options}
        if model not in self.no_think:
            body["think"] = False
        reply = {"calls": [], "content": "", "thinking": "", "first_s": None, "total_s": None}
        start = time.monotonic()
        request = urllib.request.Request(self.host + "/api/chat", data=json.dumps(body).encode(),
                                         headers={"Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(request, timeout=timeout) as stream:
                for line in stream:
                    if not line.strip():
                        continue
                    piece = json.loads(line)
                    if piece.get("error"):
                        reply["error"] = piece["error"]
                        break
                    message = piece.get("message", {})
                    if reply["first_s"] is None and (message.get("content") or message.get("thinking") or message.get("tool_calls")):
                        reply["first_s"] = time.monotonic() - start
                    reply["content"] += message.get("content") or ""
                    reply["thinking"] += message.get("thinking") or ""
                    for call in message.get("tool_calls") or []:
                        function = call.get("function", {})
                        reply["calls"].append({"name": function.get("name"), "args": function.get("arguments")})
                    if piece.get("done"):
                        for key in ("eval_count", "prompt_eval_count", "load_duration", "done_reason"):
                            if key in piece:
                                reply[key] = piece[key]
        except urllib.error.HTTPError as e:
            text = e.read().decode(errors="replace")[:300]
            if e.code == 400 and "think" in text and model not in self.no_think:
                # A model with no thinking to turn off refuses the field: ask again without it.
                self.no_think.add(model)
                return self.ask(model, messages, offered, options, timeout)
            reply["error"] = "HTTP %d: %s" % (e.code, text)
        except (OSError, ValueError) as e:
            reply["error"] = "%s: %s" % (type(e).__name__, e)
        reply["total_s"] = time.monotonic() - start
        return reply


def card_memory():
    """The memory in use on each AMD card, in bytes: everything on it, the game too."""
    used = {}
    for path in sorted(glob.glob("/sys/class/drm/card*/device/mem_info_vram_used")):
        try:
            with open(path) as f:
                used[path.split("/")[4]] = int(f.read())
        except (OSError, ValueError):
            pass
    return used


def as_ollama_tools(names, tools):
    return [{"type": "function", "function": {"name": n, "description": tools[n]["description"],
                                              "parameters": tools[n]["parameters"]}} for n in names]


def conversation(case, system):
    return [{"role": "system", "content": system}, {"role": "user", "content": case["say"]}] + case.get("messages", [])


# --- the summary -------------------------------------------------------------------------------

def percent(rows):
    return "%.1f" % (100 * sum(r["outcome"] == "correct" for r in rows) / len(rows)) if rows else "-"


def quantile(values, q):
    values = sorted(values)
    if not values:
        return None
    return values[min(len(values) - 1, int(round(q * (len(values) - 1))))]


def seconds(value):
    return "-" if value is None else "%.2f" % value


def gib(value):
    return "-" if not value else "%.2f" % (value / 2**30)


def table(head, rows):
    lines = ["| " + " | ".join(head) + " |", "|" + "|".join(" --- " for _ in head) + "|"]
    return "\n".join(lines + ["| " + " | ".join(str(c) for c in row) + " |" for row in rows]) + "\n"


def summary(results, memory, tools, info):
    models = list(dict.fromkeys(r["model"] for r in results))
    by = {m: [r for r in results if r["model"] == m] for m in models}
    out = ["# Kavat's model, tested\n",
           "%s. Ollama %s at %s. %d cases, %d runs each, temperature %s, seeds from %d, num_ctx %d.\n" % (
               info["when"], info["version"], info["host"], info["cases"], info["runs"], info["temperature"],
               info["seed"], info["num_ctx"]),
           "Correct is the share of replies that were right, over every run. Latency is in seconds; memory in GiB.\n"]

    rows = []
    for m in models:
        rs = by[m]
        firsts = [r["first_s"] for r in rs if r["first_s"] is not None]
        totals = [r["total_s"] for r in rs if r["outcome"] != "error"]
        mem = memory.get(m, {})
        rows.append([m, percent(rs)] + [percent([r for r in rs if r["kind"] == k]) for k in KINDS] + [
            seconds(statistics.median(firsts) if firsts else None), seconds(quantile(firsts, 0.95)),
            seconds(statistics.median(totals) if totals else None), seconds(quantile(totals, 0.95)),
            gib(mem.get("size_vram")), gib(mem.get("card_after")), sum(r["harmful"] for r in rs)])
    out.append(table(["Model", "Correct %", "Tool %", "Answer %", "No tool fits %", "Refuse %", "First, median",
                      "First, p95", "Whole, median", "Whole, p95", "VRAM, Ollama", "VRAM, card", "Wrong and harmful"], rows))
    out.append("Wrong and harmful counts the wrong calls to a tool above Reversible. VRAM, card is everything on the card "
               "with the model loaded: with a game open, the game is in it.\n")

    for title, key, order in (("By category", "category", None), ("By way of asking", "style", STYLES)):
        values = order or list(dict.fromkeys(r[key] for r in results))
        rows = [[v, sum(1 for r in by[models[0]] if r[key] == v and r["run"] == by[models[0]][0]["run"])] +
                [percent([r for r in by[m] if r[key] == v]) for m in models] for v in values]
        out.append("## %s\n\n" % title + table([key.capitalize(), "Cases"] + models, rows))

    rows = [[o] + [sum(r["outcome"] == o for r in by[m]) for m in models] for o in OUTCOMES]
    out.append("## What the replies were\n\n" + table(["Outcome"] + models, rows))

    rows = []
    for m in models:
        runs = sorted({r["run"] for r in by[m]})
        rows.append([m] + [percent([r for r in by[m] if r["run"] == run]) for run in runs])
    out.append("## By run\n\n" + table(["Model"] + ["Run %d" % (i + 1) for i in range(len(rows[0]) - 1)], rows))

    rows = []
    for m in models:
        mem = memory.get(m, {})
        rows.append([m, seconds(mem.get("load_s")), gib(mem.get("size")), gib(mem.get("size_vram")),
                     gib(mem.get("card_before")), gib(mem.get("card_after"))])
    out.append("## Memory\n\n" + table(["Model", "Load, s", "Size, Ollama", "VRAM, Ollama", "Card before", "Card after"], rows))
    return "\n".join(out)


# --- main --------------------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="The test that confirms Kavat's model.")
    parser.add_argument("--models", help="Ollama tags, with commas between them; the candidates when left out")
    parser.add_argument("--runs", type=int, default=4, help="how many times each case is asked (4)")
    parser.add_argument("--cases", default=os.path.join(HERE, "cases.jsonl"))
    parser.add_argument("--tools", default=os.path.join(HERE, "tools.json"))
    parser.add_argument("--system", default=os.path.join(HERE, "system.txt"))
    parser.add_argument("--limit", type=int, help="only the first cases, this many")
    parser.add_argument("--out", help="the folder for results.jsonl and summary.md; out/agent-bench/<date> when left out")
    parser.add_argument("--host", default="http://127.0.0.1:11434")
    parser.add_argument("--temperature", type=float, default=0.2)
    parser.add_argument("--seed", type=int, default=1000, help="the first run's seed; each run adds one (1000)")
    parser.add_argument("--num-ctx", type=int, default=8192)
    parser.add_argument("--timeout", type=float, default=120, help="seconds a reply may take (120)")
    parser.add_argument("--dry-run", action="store_true", help="check the catalogue and the cases, with no network")
    args = parser.parse_args()

    tools, catalogue = load_tools(args.tools)
    cases = load_cases(args.cases)
    with open(args.system) as f:
        system = f.read().strip()
    problems = check_catalogue(tools, catalogue) + check_cases(cases, tools)
    if not system:
        problems.append("system.txt is empty")
    if problems:
        raise Bad("\n".join(problems) + "\n%d problems." % len(problems))
    if args.limit:
        cases = cases[:args.limit]
    models = [m.strip() for m in args.models.split(",") if m.strip()] if args.models else CANDIDATES

    if args.dry_run:
        print("%d tools: %s" % (len(tools), ", ".join("%d %s" % (sum(t["level"] == l for t in tools.values()), l) for l in LEVELS)))
        print("%d cases:" % len(cases))
        for key, values in (("category", None), ("kind", KINDS), ("style", STYLES)):
            get = (lambda c: c["expect"]["kind"]) if key == "kind" else (lambda c, key=key: c[key])
            values = values or list(dict.fromkeys(get(c) for c in cases))
            print("  by %s: %s" % (key, ", ".join("%s %d" % (v, sum(get(c) == v for c in cases)) for v in values)))
        used = {n for c in cases for n in c["offer"]}
        expected = {call["tool"] for c in cases if c["expect"]["kind"] == "tool" for call in c["expect"]["calls"]}
        print("  tools never expected: %s" % (", ".join(sorted(set(tools) - expected)) or "none"))
        print("  tools never on offer: %s" % (", ".join(sorted(set(tools) - used)) or "none"))
        for model in models:
            if model.startswith(PLACEHOLDER):
                print("warning: %s is not a tag yet: fill it in, in CANDIDATES, before a run" % model)
        print("The catalogue and the cases are sound. Nothing was asked of a model.")
        return

    waiting = [m for m in models if m.startswith(PLACEHOLDER)]
    if waiting:
        raise Bad("%s is not an Ollama tag: the report gives none for that model.\n"
                  "Write its tag in CANDIDATES, in %s, or name the models with --models." % (", ".join(waiting), __file__))
    ollama = Ollama(args.host)
    version = ollama.version()
    have = ollama.pulled()
    missing = [m for m in models if m not in have]
    if missing:
        raise Bad("Not pulled:\n" + "\n".join("  ollama pull " + m for m in missing))

    when = time.strftime("%Y-%m-%d %H:%M")
    out = args.out or os.path.join(REPO, "out", "agent-bench", time.strftime("%Y%m%d-%H%M"))
    os.makedirs(out, exist_ok=True)
    results, memory = [], {}
    with open(os.path.join(out, "results.jsonl"), "w") as log:
        for model in models:
            for other in models:
                ollama.unload(other)
            time.sleep(2)
            before = card_memory()
            try:
                load_s = ollama.load(model, args.num_ctx)
            except (OSError, ValueError) as e:
                raise Bad("%s does not load: %s" % (model, e))
            after = card_memory()
            memory[model] = dict(ollama.memory(model), load_s=load_s, card_before=sum(before.values()),
                                 card_after=sum(after.values()), cards=after)
            log.write(json.dumps({"model": model, "memory": memory[model]}) + "\n")
            print("%s: loaded in %.1f s, %s GiB of VRAM by Ollama's count, %s GiB in use on the card" % (
                model, load_s, gib(memory[model]["size_vram"]), gib(memory[model]["card_after"])), flush=True)
            for run in range(args.runs):
                options = {"temperature": args.temperature, "seed": args.seed + run, "num_ctx": args.num_ctx,
                           "num_predict": 512}
                right = 0
                for case in cases:
                    reply = ollama.ask(model, conversation(case, system), as_ollama_tools(case["offer"], tools),
                                       options, args.timeout)
                    outcome, why = score(case, tools, reply)
                    right += outcome == "correct"
                    first = reply["calls"][0]["name"] if reply["calls"] else None
                    row = {"model": model, "run": run, "seed": options["seed"], "id": case["id"],
                           "category": case["category"], "style": case["style"], "kind": case["expect"]["kind"],
                           "outcome": outcome, "why": why,
                           "harmful": outcome != "correct" and first in tools and tools[first]["level"] in LEVELS[2:],
                           "first_s": reply["first_s"], "total_s": reply["total_s"], "calls": reply["calls"],
                           "content": reply["content"], "thinking": reply["thinking"],
                           "eval_count": reply.get("eval_count"), "prompt_eval_count": reply.get("prompt_eval_count")}
                    results.append(row)
                    log.write(json.dumps(row, ensure_ascii=False) + "\n")
                    log.flush()
                print("  run %d: %d of %d" % (run + 1, right, len(cases)), flush=True)
            ollama.unload(model)

    info = {"when": when, "version": version, "host": args.host, "cases": len(cases), "runs": args.runs,
            "temperature": args.temperature, "seed": args.seed, "num_ctx": args.num_ctx}
    text = summary(results, memory, tools, info)
    with open(os.path.join(out, "summary.md"), "w") as f:
        f.write(text)
    print("\n" + text)
    print("Written to %s" % out)


if __name__ == "__main__":
    try:
        main()
    except Bad as e:
        sys.exit("run.py: %s" % e)
    except KeyboardInterrupt:
        sys.exit("run.py: stopped")
