"""Local demonstration upstream for Garlic HTTP exchange tests.
Run: python examples/upstream.py
"""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import time

class Handler(BaseHTTPRequestHandler):
    def send(self, code, payload):
        data = json.dumps(payload, ensure_ascii=False).encode('utf-8')
        self.send_response(code)
        self.send_header('Content-Type', 'application/json; charset=utf-8')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path.startswith('/slow'):
            time.sleep(7)
        self.send(200, {'method': 'GET', 'path': self.path, 'message': 'hello from upstream'})

    def do_POST(self):
        n = int(self.headers.get('Content-Length', '0'))
        if n > 1024 * 1024:
            self.send(413, {'error': 'payload too large'})
            return
        body = self.rfile.read(n)
        self.send(200, {'method': 'POST', 'body': body.decode('utf-8', errors='replace')})

ThreadingHTTPServer(('127.0.0.1', 9000), Handler).serve_forever()
