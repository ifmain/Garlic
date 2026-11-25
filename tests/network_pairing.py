"""Run on an isolated host with loopback TCP 443 available. Uses a throwaway client profile."""
import contextlib
import json
import os
import ssl
import subprocess
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from cryptography import x509
from cryptography.hazmat.primitives.serialization import Encoding
from windows_module_reload import certificates

def run(core, root):
    issuer, network = root/"issuer", root/"network"
    for folder in (issuer, network):
        folder.mkdir()
        certificates(folder, folder.name, ("localhost",))
    der = x509.load_pem_x509_certificate((network/"ca.pem").read_bytes()).public_bytes(Encoding.DER)
    requests, data_calls = [], []
    class Enrollment(BaseHTTPRequestHandler):
        def do_GET(self):
            requests.append((self.path, self.headers.get("Authorization"), self.headers.get("Cookie")))
            self.send_response(200)
            self.send_header('Content-Length',str(len(der)))
            self.end_headers()
            self.wfile.write(der)
        def log_message(self,*args): pass
    class Gateway(BaseHTTPRequestHandler):
        def do_GET(self):
            data_calls.append(self.headers.get("Authorization"))
            self.send_response(401)
            self.send_header("Content-Length",str(len(b'{"error":"fixture"}')))
            self.end_headers()
            self.wfile.write(b'{"error":"fixture"}')
        def log_message(self,*args): pass
    def serve(stack, folder, port, handler):
        server = stack.enter_context(ThreadingHTTPServer(("127.0.0.1",port),handler))
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ctx.load_cert_chain(folder/"cert.pem",folder/"key.pem")
        server.socket = ctx.wrap_socket(server.socket,server_side=True)
        threading.Thread(target=server.serve_forever,daemon=True).start()
        stack.callback(server.shutdown)
        return server.server_port
    def connect(label, env):
        result = subprocess.run([core,"connect",label,"--key-stdin"],env=env,input="fixture",text=True,capture_output=True,timeout=20)
        return result.stderr
    env = dict(os.environ,GARLIC_CA_CERT=str(issuer/"ca.pem"),GARLIC_CLIENT_CONFIG=str(root/"client.json"))
    with contextlib.ExitStack() as data, contextlib.ExitStack() as web:
        target = "https://127.0.0.1:"+str(serve(data,network,0,Gateway))
        serve(web,issuer,443,Enrollment)
        # Without an independently trusted bootstrap identity no token may leave the client.
        untrusted=dict(env)
        del untrusted["GARLIC_CA_CERT"]
        assert "401" not in connect(target,untrusted)
        assert not requests and not data_calls and not (root/"client.pairing.json").exists()
        response = connect(target,env)
        assert "401" in response, response
        assert requests==[("/.well-known/garlic/ca.der",None,None)] and data_calls==["Bearer fixture"]
        cache=json.loads((root/"client.pairing.json").read_text())
        assert list(cache.values())==[list(der)]
        # 0.9.18 stored the authenticated network directory but no CA file.
        legacy = root/"legacy"
        legacy.mkdir()
        (legacy/"client.network.json").write_text(json.dumps({"version":1,"nodes":[
            {"role":"ru","url":"https://localhost:8443"},{"role":"exit","url":target}]}))
        migrated = dict(env,GARLIC_CLIENT_CONFIG=str(legacy/"client.json"))
        response = connect(target,migrated)
        assert "401" in response, response
        assert list(json.loads((legacy/"client.pairing.json").read_bytes())) == ["https://localhost/.well-known/garlic/ca.der"]
        web.close()
        response = connect(target,env)
        assert "401" in response and len(data_calls)==3, response
        print("PASS first pairing, rejection of unknown bootstrap CA, credential isolation, offline CA reuse")
if __name__=="__main__":
    with tempfile.TemporaryDirectory() as tmp: run(str(Path(sys.argv[1]).resolve()),Path(tmp))
