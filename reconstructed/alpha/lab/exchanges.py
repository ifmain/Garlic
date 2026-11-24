"""Force each embedded protocol and check payload and the negotiated mode."""
import json
import os
from pathlib import Path
import socket
import socketserver
import struct
import subprocess
from threading import Thread
import time
from urllib.request import build_opener, ProxyHandler


def exact(sock, size):
    data = bytearray()
    while len(data)<size:
        part=sock.recv(size-len(data))
        if not part:
            raise EOFError("short echo")
        data.extend(part)
    return bytes(data)


def verify(binary, folder, env, password, url):
    class Echo(socketserver.BaseRequestHandler):
        def handle(self):
            while data := self.request.recv(16384):
                self.request.sendall(data)
    results=[]
    with socketserver.ThreadingTCPServer(("127.0.0.1",0),Echo) as echo:
        thread=Thread(target=echo.serve_forever,daemon=True)
        thread.start()
        try:
            for version in ("2","3","4"):
                with socket.socket() as s:
                    s.bind(("127.0.0.1",0))
                    port=s.getsockname()[1]
                e=dict(env,GARLIC_ALPHA_PROTOCOLS=version,GARLIC_CLIENT_CONFIG=str(folder/"client.json"),
                       GARLIC_ROUTE_MODE="direct",GARLIC_STORAGE_URLS=url,GARLIC_CONTROL_URLS="",
                       GARLIC_PROXY_BIND=f"127.0.0.1:{port}")
                subprocess.run([binary,"login",url,"benchmark","--password-stdin"],cwd=folder,env=e,
                               input=password+"\n",text=True,stdout=subprocess.DEVNULL,check=True)
                p=subprocess.Popen([binary,"proxy"],cwd=folder,env=e,stdout=subprocess.DEVNULL)
                try:
                    api=build_opener(ProxyHandler({}))
                    for _ in range(100):
                        try:
                            api.open(f"http://127.0.0.1:{port}/status",timeout=.5).close()
                            break
                        except OSError:
                            if p.poll() is not None: raise RuntimeError("client stopped")
                            time.sleep(.05)
                    payload=os.urandom(250000)
                    with socket.create_connection(("127.0.0.1",port),timeout=20) as sock:
                        sock.sendall(b"\x05\x01\x00")
                        assert exact(sock,2)==b"\x05\x00"
                        sock.sendall(b"\x05\x01\x00\x01\x7f\x00\x00\x01"+struct.pack("!H",echo.server_address[1]))
                        assert exact(sock,10)[:2]==b"\x05\x00"
                        sock.sendall(payload)
                        assert exact(sock,len(payload))==payload
                    with api.open(f"http://127.0.0.1:{port}/status",timeout=2) as r:
                        status=json.load(r)
                    assert status["transport"]["protocol"]=="GEX"+version
                    results.append({"negotiated":"GEX"+version,"verified_bytes":len(payload)})
                finally:
                    p.terminate()
                    p.wait(timeout=10)
        finally:
            echo.shutdown()
            thread.join()
    return results
