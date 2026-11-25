"""Q55 test update server. Throwaway code.

Serves files from a folder, with faults by path prefix:
  /f/<name>     the file
  /cut/<name>   sends the full Content-Length, half the body, then closes
Each request is appended to <folder>/access.log.
Usage: python3 -I server.py <folder> <port>
"""

import http.server
import os
import sys
import threading
import time

ROOT = sys.argv[1]
PORT = int(sys.argv[2])
LOCK = threading.Lock()


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):
        with LOCK, open(os.path.join(ROOT, "access.log"), "a") as f:
            f.write(f"{time.time():.3f} {self.path} {args[1] if len(args) > 1 else ''}\n")

    def do_GET(self):
        mode, _, name = self.path.lstrip("/").partition("/")
        path = os.path.join(ROOT, os.path.basename(name))
        if mode not in ("f", "cut") or not os.path.isfile(path):
            self.send_response(404)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        with open(path, "rb") as f:
            data = f.read()
        self.send_response(200)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        if mode == "cut":
            self.wfile.write(data[: len(data) // 2])
            self.wfile.flush()
            self.close_connection = True
            self.connection.shutdown(2)
            return
        self.wfile.write(data)


http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
