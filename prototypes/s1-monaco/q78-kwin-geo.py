"""Print the KWin window list as JSON. Throwaway Q78 tool. Never merge."""
import json, os, sys, tempfile, time
import gi
gi.require_version("Gio", "2.0")
from gi.repository import Gio, GLib

NAME = "org.q78.Sink"
XML = """<node><interface name="org.q78.Sink"><method name="put"><arg type="s" direction="in"/></method></interface></node>"""
result = []
loop = GLib.MainLoop()

def on_call(conn, sender, path, iface, method, params, inv):
    result.append(params.unpack()[0])
    inv.return_value(None)
    loop.quit()

bus = Gio.bus_get_sync(Gio.BusType.SESSION)
node = Gio.DBusNodeInfo.new_for_xml(XML)
bus.register_object(  # deprecated, fine for a throwaway tool
    "/", node.interfaces[0], on_call, None, None)
Gio.bus_own_name_on_connection(bus, NAME, 0, None, None)
script = """
const out = workspace.windowList().map(w => ({cls: w.resourceClass, caption: w.caption, x: w.frameGeometry.x, y: w.frameGeometry.y,
  w: w.frameGeometry.width, h: w.frameGeometry.height, cx: w.clientGeometry.x, cy: w.clientGeometry.y,
  managed: w.managed, special: w.specialWindow, popup: w.popupWindow, active: w.active, pid: w.pid}));
callDBus("org.q78.Sink", "/", "org.q78.Sink", "put", JSON.stringify(out));
"""
fd, path = tempfile.mkstemp(suffix=".js")
os.write(fd, script.encode()); os.close(fd)
def call(path_, iface, method, args, rt):
    return bus.call_sync("org.kde.KWin", path_, iface, method, args, rt, 0, 5000, None)
GLib.timeout_add(200, lambda: False)
name = "q78geo%d" % os.getpid()
sid = call("/Scripting", "org.kde.kwin.Scripting", "loadScript", GLib.Variant("(ss)", (path, name)), GLib.VariantType("(i)")).unpack()[0]
call("/Scripting/Script%d" % sid, "org.kde.kwin.Script", "run", None, None)
GLib.timeout_add(3000, loop.quit)
loop.run()
call("/Scripting", "org.kde.kwin.Scripting", "unloadScript", GLib.Variant("(s)", (name,)), GLib.VariantType("(b)"))
os.unlink(path)
print(result[0] if result else "[]")
