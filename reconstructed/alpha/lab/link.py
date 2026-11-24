"""Fault injection on a fixed localhost TLS link. No system network changes."""
from contextlib import contextmanager
import select
import socket
from socketserver import BaseRequestHandler, ThreadingTCPServer
from threading import Lock, Thread
import time


@contextmanager
def link(origin_port, profile):
    lock = Lock()
    drop_next = [False]
    def lose_response():
        with lock:
            drop_next[0] = True
    class Handler(BaseRequestHandler):
        def handle(self):
            with lock:
                drop = drop_next[0]
                drop_next[0] = False
            try:
                with socket.create_connection(("127.0.0.1",origin_port),timeout=3) as peer:
                    # Allow the TLS handshake. The controlled loss of the application
                    # response is injected by the HTTP client after request transmission.
                    fds = (self.request,peer)
                    while True:
                        ready,_,_ = select.select(fds,[],[],5)
                        if not ready:
                            return
                        for src in ready:
                            buf = src.recv(16384)
                            if not buf:
                                return
                            if profile == "delayed":
                                time.sleep(.015)
                            if profile == "rate128KiB":
                                time.sleep(len(buf)/131072)
                            if drop:
                                time.sleep(1)
                                return
                            (peer if src is self.request else self.request).sendall(buf)
            except OSError:
                pass
    class Listener(ThreadingTCPServer):
        daemon_threads = True
    with Listener(("127.0.0.1",0),Handler) as server:
        t=Thread(target=server.serve_forever,daemon=True)
        t.start()
        try:
            yield server.server_address[1],lose_response
        finally:
            server.shutdown()
            t.join()
