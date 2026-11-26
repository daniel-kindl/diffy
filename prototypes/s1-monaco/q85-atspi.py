"""Q85: read the AT-SPI tree of option B, as Orca reads it. Throwaway. Never merge.

Usage: python3 -I q85-atspi.py <label> <pid> <out json>
Waits for the document title "Q85 ready <label>", then records the focused
object, its ancestors and the text around the caret.
"""
import json
import sys
import time

import gi
gi.require_version("Atspi", "2.0")
from gi.repository import Atspi

LABEL, PID, OUT = sys.argv[1], int(sys.argv[2]), sys.argv[3]
Atspi.init()


def states(a):
    s = a.get_state_set()
    return sorted(Atspi.StateType(i).value_nick for i in range(int(Atspi.StateType.LAST_DEFINED)) if s.contains(Atspi.StateType(i)))


def info(a):
    d = {"role": a.get_role_name(), "name": a.get_name(), "description": a.get_description(), "states": states(a)}
    try:
        d["attributes"] = dict(a.get_attributes() or {})
    except Exception as e:
        d["attributes_error"] = str(e)
    return d


def walk(a, depth=0, limit=40):
    yield a, depth
    if depth >= limit:
        return
    for i in range(a.get_child_count()):
        c = a.get_child_at_index(i)
        if c is not None:
            yield from walk(c, depth + 1, limit)


def find_app():
    desk = Atspi.get_desktop(0)
    for i in range(desk.get_child_count()):
        app = desk.get_child_at_index(i)
        try:
            if app is not None and app.get_process_id() == PID:
                return app
        except Exception:
            pass
    return None


def text_facts(a):
    if a.get_text_iface() is None:
        return None
    T = Atspi.Text
    n = T.get_character_count(a)
    caret = T.get_caret_offset(a)
    line = T.get_string_at_offset(a, max(caret, 0), Atspi.TextGranularity.LINE)
    return {"character_count": n, "caret_offset": caret, "text": T.get_text(a, 0, min(n, 2000)),
            "line_at_caret": {"text": line.content, "start": line.start_offset, "end": line.end_offset}}


result = {"label": LABEL, "pid": PID}
end = time.time() + 120
doc = app = None
while time.time() < end:
    app = find_app()
    if app is not None:
        for a, _ in walk(app, limit=12):
            if a.get_role_name() == "document web" and (a.get_name() or "").startswith(f"Q85 ready {LABEL}"):
                doc = a
                break
    if doc is not None:
        break
    time.sleep(0.5)
result["app_found"] = app is not None
result["ready"] = doc is not None
if doc is not None:
    time.sleep(0.5)
    focused = [a for a, _ in walk(doc) if "focused" in states(a)]
    result["focused"] = []
    for f in focused:
        d = info(f)
        d["text"] = text_facts(f)
        chain, p = [], f.get_parent()
        while p is not None and p != doc:
            chain.append({"role": p.get_role_name(), "name": p.get_name()})
            p = p.get_parent()
        d["ancestors"] = chain
        result["focused"].append(d)
    # Whole-document scan: objects that name a change type, and live regions.
    words = ("added", "removed", "inserted", "deleted", "insert", "delete")
    hits, live = [], []
    for a, _ in walk(doc):
        name = (a.get_name() or "") + " " + (a.get_description() or "")
        if any(w in name.lower() for w in words):
            hits.append(info(a))
        attrs = {}
        try:
            attrs = dict(a.get_attributes() or {})
        except Exception:
            pass
        if "live" in attrs or "container-live" in attrs:
            d = info(a)
            d["text"] = text_facts(a)
            live.append(d)
    result["change_type_named"] = hits[:20]
    result["live_regions"] = live[:20]
    result["document"] = info(doc)
with open(OUT, "w", encoding="utf-8") as fh:
    json.dump(result, fh, indent=2, ensure_ascii=False)
print(json.dumps({"ready": result["ready"], "focused": len(result.get("focused", []))}))
