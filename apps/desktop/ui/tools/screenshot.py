"""Render the built UI offscreen with WebKit and save a PNG (a developer tool).

The page talks to Rust through Tauri. Here Tauri is replaced by a small stub with
fixed data, so only the page itself is shown: its layout, its theme and its
Monaco editors. It is not the app and it does not test the Rust commands.

Needs PyGObject with GTK 3 and WebKit2 4.1, and a display (X11 or XWayland).

    npm run build
    (cd dist && python3 -m http.server 8765 --bind 127.0.0.1) &
    DISPLAY=:0 GDK_BACKEND=x11 python3 tools/screenshot.py http://127.0.0.1:8765/ out.png [light|dark] [javascript to run after load]
"""
import gi, sys, json
gi.require_version('Gtk', '3.0'); gi.require_version('WebKit2', '4.1')
from gi.repository import Gtk, WebKit2, GLib, Gdk, GdkPixbuf
import cairo

url, out, theme = sys.argv[1], sys.argv[2], (sys.argv[3] if len(sys.argv) > 3 else "")
after_load = sys.argv[4] if len(sys.argv) > 4 else ""  # JavaScript to run, for example a click
STUB = """
(function () {
  // A longer file, so the unchanged runs can collapse: line 20 changes and one line is added after line 33.
  const lines = Array.from({ length: 120 }, (_, i) => "    let value_" + (i + 1) + " = " + (i + 1) + ";");
  const base = lines.join("\\n") + "\\n";
  const headLines = lines.slice();
  headLines[19] = "    let value_20 = 200;";
  headLines.splice(33, 0, "    let extra = 0;");
  const head = headLines.join("\\n") + "\\n";
  const data = {
    current_comparison: { base, head, sides: [
      { label: "HEAD:src/main.rs", encoding: "UTF-8", first_error: null },
      { label: "src/main.rs", encoding: "UTF-8", first_error: null }],
      hunks: [ { base: {start: 19, end: 20}, head: {start: 19, end: 20} },
               { base: {start: 33, end: 33}, head: {start: 33, end: 34} } ] },
    repository_files: { files: [
      { path: "src/main.rs", status: "modified", generated: false, reviewed: true },
      { path: "src/lib.rs", status: "added", generated: false, reviewed: false },
      { path: "README.md", status: "modified", generated: false, reviewed: false } ],
      total: 4, hidden: 1, generated: 1 },
    review_findings: [ { id: 0, path: "src/main.rs", start_line: 3, end_line: 3, severity: "major",
      category: "correctness", notes: "b changed to 3: is that intended?", resolved: false } ],
  };
  window.__TAURI_INTERNALS__ = {
    // Keep the callbacks, so a test can fire an event: window.__fire().
    transformCallback: (cb) => { window.__callbacks = window.__callbacks || []; window.__callbacks.push(cb); return window.__callbacks.length; },
    unregisterCallback: () => {},
    convertFileSrc: (p) => p,
    invoke: async (cmd) => {
      if (cmd === 'current_comparison') {
        // The second load is marked, and window.__other = true makes it another comparison.
        window.__loads = (window.__loads || 0) + 1;
        if (window.__loads > 1) {
          const c = JSON.parse(JSON.stringify(data.current_comparison));
          c.sides[0].encoding = 'REFRESHED';
          if (window.__other) c.sides[1].label = 'src/other.rs';
          return c;
        }
      }
      if (cmd in data) return data[cmd];
      if (cmd.startsWith("plugin:event")) return 0;
      return null;
    },
  };
  window.__fire = () => (window.__callbacks || []).forEach((cb) => cb({ event: 'diffy-changed', id: 0, payload: null }));
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  """ + ("try { localStorage.setItem('diffy.theme','%s'); } catch (e) {}" % theme if theme else "") + """
})();
"""
win = Gtk.OffscreenWindow(); win.set_default_size(1280, 720)
ucm = WebKit2.UserContentManager()
ucm.add_script(WebKit2.UserScript(STUB, WebKit2.UserContentInjectedFrames.ALL_FRAMES, WebKit2.UserScriptInjectionTime.START, None, None))
view = WebKit2.WebView.new_with_user_content_manager(ucm)
win.add(view); win.show_all()
view.load_uri(url)

def shoot():
    if after_load:
        view.run_javascript(after_load, None, None, None)
        GLib.timeout_add(4000, shoot_now)
        return False
    return shoot_now()


def shoot_now():
    def done(v, res):
        surf = v.get_snapshot_finish(res)
        surf.write_to_png(out); print("saved", out); Gtk.main_quit()
    view.get_snapshot(WebKit2.SnapshotRegion.FULL_DOCUMENT, WebKit2.SnapshotOptions.NONE, None, done)
    return False
GLib.timeout_add(5000, shoot)
GLib.timeout_add(30000, Gtk.main_quit)
Gtk.main()
