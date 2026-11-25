"""Run stock Windows GEX2 and rebuilt GEX3/4 against a local modular exit."""
import base64
import contextlib
import datetime as dt
import ipaddress
import json
import os
from pathlib import Path
import secrets
import select
import socket
import socketserver
import ssl
import struct
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request
import uuid

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec, ed25519
from cryptography.x509.oid import NameOID, ExtendedKeyUsageOID

RUST = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get("GARLIC_FIXTURE_BIN", RUST / "target/debug")).resolve()
CORE = Path(os.environ.get("GARLIC_FIXTURE_CLIENT", BIN/"garlic-internal-network.exe")).resolve()
STOCK = Path(os.environ.get("GARLIC_STOCK_CLIENT", CORE))
DEFAULT = json.loads((RUST / "crates/coordinator/src/exchanges/modules-v1.json").read_bytes())


def certificates(folder, ca_name="fixture CA", dns_names=()):
    now = dt.datetime.now(dt.timezone.utc)
    def build(key, name, issuer, signer, ca):
        cert = (x509.CertificateBuilder().subject_name(name).issuer_name(issuer)
                .public_key(key.public_key()).serial_number(x509.random_serial_number())
                .not_valid_before(now - dt.timedelta(minutes=5)).not_valid_after(now + dt.timedelta(days=1))
                .add_extension(x509.BasicConstraints(ca=ca, path_length=None), critical=True)
                .add_extension(x509.SubjectKeyIdentifier.from_public_key(key.public_key()), False)
                .add_extension(x509.AuthorityKeyIdentifier.from_issuer_public_key(signer.public_key()), False)
                .add_extension(x509.KeyUsage(True,False,False,False,False,ca,ca,False,False),True))
        if not ca:
            cert = cert.add_extension(x509.SubjectAlternativeName([x509.IPAddress(ipaddress.ip_address("127.0.0.1")), *(x509.DNSName(n) for n in dns_names)]), False)
            cert = cert.add_extension(x509.ExtendedKeyUsage([ExtendedKeyUsageOID.SERVER_AUTH]), False)
        return cert.sign(signer, hashes.SHA256()).public_bytes(serialization.Encoding.PEM)
    root, leaf = (ec.generate_private_key(ec.SECP256R1()) for _ in range(2))
    name = lambda n: x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, n)])
    (folder/"ca.pem").write_bytes(build(root, name(ca_name), name(ca_name), root, True))
    (folder/"cert.pem").write_bytes(build(leaf, name("localhost"), name(ca_name), root, False))
    (folder/"key.pem").write_bytes(leaf.private_bytes(serialization.Encoding.PEM, serialization.PrivateFormat.PKCS8, serialization.NoEncryption()))


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def wait(check):
    until = time.monotonic() + 20
    last = None
    while time.monotonic() < until:
        try:
            if check():
                return
        except OSError as exc:
            last = exc
        time.sleep(.1)
    raise RuntimeError("fixture readiness timed out") from last


@contextlib.contextmanager
def delayed_gateway(upstream_port, gate):
    """Expose the ingress after a direct stream attempt, regardless of probe timing."""
    class Forward(socketserver.BaseRequestHandler):
        def handle(self):
            if not gate.wait(12):
                return
            with contextlib.suppress(OSError), socket.create_connection(("127.0.0.1",upstream_port),5) as upstream:
                sockets = [self.request,upstream]
                while True:
                    ready,_,_ = select.select(sockets,[],[],20)
                    if not ready:
                        return
                    for source in ready:
                        data = source.recv(65536)
                        if not data:
                            return
                        sockets[1-sockets.index(source)].sendall(data)
    with socketserver.ThreadingTCPServer(("127.0.0.1",0),Forward) as server:
        server.daemon_threads = True
        threading.Thread(target=server.serve_forever,daemon=True).start()
        try:
            yield server.server_address[1]
        finally:
            gate.set()
            server.shutdown()


class Echo(socketserver.BaseRequestHandler):
    def handle(self):
        with contextlib.suppress(ConnectionResetError):
            self.echo()

    def echo(self):
        while data := self.request.recv(65536):
            self.request.sendall(data)


def roundtrip(conn):
    sent = secrets.token_bytes(40000)
    conn.sendall(sent)
    received = bytearray()
    while len(received) < len(sent):
        chunk = conn.recv(len(sent) - len(received))
        if not chunk:
            raise EOFError("echo stream closed")
        received.extend(chunk)
    assert received == sent


def custom_lane(port, ctx, token, enabled):
    with socket.create_connection(("127.0.0.1", port), 4) as tcp:
        with ctx.wrap_socket(tcp, server_hostname="127.0.0.1") as ws, ws.makefile("rb") as reader:
            key = base64.b64encode(secrets.token_bytes(16)).decode()
            ws.sendall((f"GET /v1/exchange/{uuid.uuid4()} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n"
                        f"Upgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\n"
                        f"Sec-WebSocket-Key: {key}\r\nAuthorization: Bearer {token}\r\n"
                        "x-garlic-exchange-version: 128\r\n\r\n").encode())
            status = reader.readline().split()[1]
            if not enabled:
                assert status == b"400"
                return
            assert status == b"101"
            headers = []
            while line := reader.readline():
                if line == b"\r\n": break
                headers.append(line.lower().strip())
            assert b"x-garlic-exchange-version: 128" in headers
            prefix = b"GEX4\0\0GEX3"
            packet = prefix + struct.pack(">QQII", 1, 0, 0, 0)
            mask = secrets.token_bytes(4)
            ws.sendall(bytes([0x82, 0x80 | len(packet)]) + mask +
                       bytes(b ^ mask[i % 4] for i, b in enumerate(packet)))
            assert reader.read(2) == bytes([0x82, len(packet)])
            assert reader.read(len(packet)) == prefix + struct.pack(">QQII", 0, 1, 0, 0)


def test(folder):
    certificates(folder)
    manifest = folder/"modules.json"
    def publish(modules):
        tmp = manifest.with_suffix(".new")
        tmp.write_text(json.dumps({"abi": 1, "modules": modules}))
        os.replace(tmp, manifest)
    publish(DEFAULT["modules"])
    port = free_port()
    origin = f"https://127.0.0.1:{port}"
    env = dict(os.environ, GARLIC_STORAGE_BIND=f"127.0.0.1:{port}", GARLIC_PUBLIC_URL=origin,
               GARLIC_DIRECT_ONLY="1", GARLIC_DEV_MODE="1", GARLIC_DEV_ALLOW_PRIVATE_PROXY_TARGETS="1",
               GARLIC_NODE_LABEL="fixture", GARLIC_WORKERS="1", GARLIC_PROTOCOL_MODULES=str(manifest),
               GARLIC_RECORD_PROFILE_FILE=str(folder/"record-profile.json"),
               GARLIC_STORAGE_STATE_FILE=str(folder/"state.json"), GARLIC_NODE_IDENTITY_FILE=str(folder/"identity.json"),
               GARLIC_AUTH_FILE=str(folder/"users.json"), GARLIC_INFRA_KEYS_FILE=str(folder/"keys.json"),
               GARLIC_CA_CERT=str(folder/"ca.pem"), GARLIC_TLS_CERT=str(folder/"cert.pem"), GARLIC_TLS_KEY=str(folder/"key.pem"),
               GARLIC_CLIENT_CONFIG=str(folder/"client.json"), APPDATA=str(folder), LOCALAPPDATA=str(folder),
               GARLIC_ROUTE_MODE="direct", GARLIC_STORAGE_URLS=origin, GARLIC_CONTROL_URLS="")
    password = secrets.token_urlsafe(30)
    def run(exe, *args, secret=None):
        subprocess.run([exe,*args], cwd=folder, env=env, input=secret, text=True, stdout=subprocess.DEVNULL, check=True, timeout=25)
    auth = BIN/"garlic-auth.exe"
    run(auth, "keys","init",env["GARLIC_INFRA_KEYS_FILE"],"fixture")
    run(auth, "users","init",env["GARLIC_AUTH_FILE"])
    run(auth, "users","add",env["GARLIC_AUTH_FILE"],"fixture","--password-stdin",secret=password)
    run(auth, "identity","challenge",env["GARLIC_NODE_IDENTITY_FILE"],env["GARLIC_INFRA_KEYS_FILE"],"fixture")
    ctx = ssl.create_default_context(cafile=env["GARLIC_CA_CERT"])
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), urllib.request.HTTPSHandler(context=ctx))
    def request(path, body=None, token=None):
        headers={"Content-Type":"application/json"}
        if token:
            headers["Authorization"]="Bearer "+token
        with opener.open(urllib.request.Request(origin+path, data=body, headers=headers),timeout=2) as r:
            return json.load(r)
    @contextlib.contextmanager
    def process(binary, *args):
        with (folder/(binary.stem+".log")).open("ab",buffering=0) as log:
            child=subprocess.Popen([binary,*args],cwd=folder,env=env,stdout=log,stderr=log)
            try:
                yield child
            except Exception:
                print((folder/(binary.stem+".log")).read_text(errors="replace")[-1600:])
                raise
            finally:
                child.terminate()
                child.wait(timeout=10)
    with process(BIN/"garlic-server.exe","--role","exit"), socketserver.ThreadingTCPServer(("127.0.0.1",0),Echo) as echo, contextlib.ExitStack() as cleanup:
        echo.daemon_threads = True
        threading.Thread(target=echo.serve_forever,daemon=True).start()
        cleanup.callback(echo.shutdown)
        wait(lambda: request("/v1/auth/info"))
        token=request("/v1/auth/login",json.dumps({"username":"fixture","password":password}).encode())["token"]
        if serial := os.environ.get("GARLIC_FIXTURE_ANDROID"):
            from android_module_probe import verify
            verify(serial, folder, port, password, request, token, publish, echo.server_address[1])
            return
        core_version = json.loads(subprocess.check_output([CORE,"version"],env=env))
        if core_version.get("alpha_profiles"):
            network = request("/v1/auth/info",token=token)["authority"]["owner_id"]
            publisher = ed25519.Ed25519PrivateKey.generate()
            (folder/f"client.profile-{network}.pub").write_bytes(publisher.public_key().public_bytes_raw())
            keyfile = folder/"fixture-signing.pem"
            keyfile.write_bytes(publisher.private_bytes(serialization.Encoding.PEM,
                serialization.PrivateFormat.PKCS8, serialization.NoEncryption()))
            subprocess.run([sys.executable, RUST/"tools/sign_profile.py", "--network", network,
                            "--sequence", "1", "--expires", str(int(time.time()) + 3600), "--frame-bytes",
                            "2048", "--key", keyfile, folder/"record-profile.json"], check=True)
        def active():
            info=request("/v1/auth/info",token=token)
            return info["exchange_versions"]+([4] if "gex4-tlv" in info.get("exchange_extensions",[]) else [])
        for module in DEFAULT["modules"]:
            version=module["version"]
            offered = [dict(module, record_size=2048)] if version < 4 else [module]
            publish(offered)
            wait(lambda: active()==[m["version"] for m in offered])
            client=STOCK if version==2 else CORE
            env["GARLIC_ALPHA_PROTOCOLS"]=str(version)
            run(client,"login",origin,"fixture","--password-stdin",secret=password)
            proxy=free_port()
            with process(client,"proxy","--bind",f"127.0.0.1:{proxy}"):
                def ready():
                    with socket.create_connection(("127.0.0.1",proxy),.2): return True
                wait(ready)
                with socket.create_connection(("127.0.0.1",proxy),8) as conn:
                    conn.settimeout(15)
                    target=f"127.0.0.1:{echo.server_address[1]}"
                    conn.sendall(f"CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n".encode())
                    head=conn.makefile("rb",buffering=0)
                    assert b" 200 " in head.readline()
                    while head.readline() not in (b"",b"\r\n"): pass
                    roundtrip(conn)
                    if version == 3 and core_version.get("alpha_profiles"):
                        local = urllib.request.build_opener(urllib.request.ProxyHandler({}))
                        with local.open(f"http://127.0.0.1:{proxy}/status",timeout=2) as response:
                            assert json.load(response)["transport"]["record_bytes"] == 2048
                        assert (folder/f"client.profile-{network}.json").is_file()
                        print("Alpha: signed network profile active on a verified GEX3 stream",flush=True)
                    assert f"exchange_open protocol={version} " in (folder/"garlic-server.log").read_text()
                    publish([])
                    wait(lambda: active()==[])
                    roundtrip(conn)
                    head.close()
                print(f"GEX{version}: 40000 bytes each way before and after module removal",flush=True)
        # Accept authenticated HTTPS discovery on Direct, then withhold its response.
        seen = threading.Event()
        release = threading.Event()
        class Stall(socketserver.StreamRequestHandler):
            def handle(self):
                if self.rfile.readline().startswith(b"GET /v1/auth/info "):
                    headers = []
                    while line := self.rfile.readline():
                        if line == b"\r\n": break
                        headers.append(line.lower())
                    if any(h.startswith(b"authorization: bearer ") for h in headers):
                        seen.set()
                    release.wait(12)

        publish(DEFAULT["modules"])
        wait(lambda: active()==[2,3,4])
        with socketserver.ThreadingTCPServer(("127.0.0.1",0),Stall) as stalled, contextlib.ExitStack() as stop:
            stalled.daemon_threads = True
            tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            tls.load_cert_chain(folder/"cert.pem",folder/"key.pem")
            stalled.socket = tls.wrap_socket(stalled.socket,server_side=True)
            threading.Thread(target=stalled.serve_forever,daemon=True).start()
            stop.callback(stalled.shutdown)
            stop.callback(release.set)
            direct = f"https://127.0.0.1:{stalled.server_address[1]}"
            sessions = folder/"client.sessions.json"
            saved = json.loads(sessions.read_bytes())
            saved["entries"][direct] = saved["entries"][origin].copy()
            sessions.write_text(json.dumps(saved))
            env.update(GARLIC_ROUTE_MODE="auto",GARLIC_STORAGE_URLS=direct,GARLIC_CONTROL_URLS=origin)
            proxy_port = free_port()
            client = CORE
            with process(client,"proxy","--bind",f"127.0.0.1:{proxy_port}"):
                def ready():
                    with socket.create_connection(("127.0.0.1",proxy_port),.2): return True
                wait(ready)
                start = time.monotonic()
                with socket.create_connection(("127.0.0.1",proxy_port),10) as conn:
                    conn.settimeout(15)
                    target=f"127.0.0.1:{echo.server_address[1]}"
                    conn.sendall(f"CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n".encode())
                    with conn.makefile("rb",buffering=0) as head:
                        assert b" 200 " in head.readline()
                        while head.readline() not in (b"",b"\r\n"): pass
                    elapsed=time.monotonic()-start
                    roundtrip(conn)
                assert elapsed<3 and seen.wait(2)
                with urllib.request.build_opener(urllib.request.ProxyHandler({})).open(f"http://127.0.0.1:{proxy_port}/status") as response:
                    route=json.load(response)["last_route"]
                assert route["origin"]==origin and route["via"]=="through_ru",route
                print(f"Auto: authenticated Direct stalled, ingress CONNECT in {elapsed:.2f}s, echoed 40000 bytes",flush=True)
        # Fast discovery is insufficient: the data-plane open can still time out.
        # Use anonymous loopback fixtures and real authenticated ingress for the echo.
        from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
        info = json.dumps(dict(request("/v1/auth/info", token=token), device_leases=False)).encode()
        attempted = []
        first_attempt = threading.Event()
        class DeadStream(BaseHTTPRequestHandler):
            def do_GET(self):
                self.send_response(200)
                self.end_headers()
                self.wfile.write(info)
            def do_POST(self):
                assert self.path == "/v1/infinity/streams"
                attempted.append(self.server.server_port)
                first_attempt.set()
                time.sleep(3)
            def log_message(self, *args):
                pass

        with contextlib.ExitStack() as stack:
            urls = []
            for _ in range(3):
                server = stack.enter_context(ThreadingHTTPServer(("127.0.0.1",0),DeadStream))
                threading.Thread(target=server.serve_forever,daemon=True).start()
                stack.callback(server.shutdown)
                url = f"http://127.0.0.1:{server.server_port}"
                saved["entries"][url] = saved["entries"][origin].copy()
                urls.append(url)
            delayed_port = stack.enter_context(delayed_gateway(port,first_attempt))
            delayed = f"https://127.0.0.1:{delayed_port}"
            saved["entries"][delayed] = saved["entries"][origin].copy()
            sessions.write_text(json.dumps(saved))
            run(CORE,"config","set","request_timeout_ms","1000")
            for ingress, works in [(delayed, True), (urls[-1], False)]:
                attempted.clear()
                env.update(GARLIC_STORAGE_URLS=",".join(urls[:2]),GARLIC_CONTROL_URLS=ingress)
                proxy_port = free_port()
                with process(CORE,"proxy","--bind",f"127.0.0.1:{proxy_port}"):
                    wait(ready)
                    time.sleep(.7)
                    with socket.create_connection(("127.0.0.1",proxy_port),8) as conn:
                        conn.sendall(f"CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n".encode())
                        with conn.makefile("rb",buffering=0) as head:
                            status_line = head.readline()
                            assert (b" 200 " if works else b" 502 ") in status_line, status_line
                            while head.readline() not in (b"",b"\r\n"): pass
                        if works:
                            roundtrip(conn)
                            assert len(attempted)==1, attempted
                        else:
                            assert len(set(attempted)) == 3 and server.server_port in attempted[:2], attempted
                    if works:
                        with socket.create_connection(("127.0.0.1",proxy_port),8) as conn:
                            conn.sendall(f"CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n".encode())
                            with conn.makefile("rb",buffering=0) as head:
                                assert b" 200 " in head.readline()
                                while head.readline() not in (b"",b"\r\n"): pass
                            roundtrip(conn)
                        assert len(attempted)==1, attempted
                    print("PASS stream-open timeout, ingress echo" if works else "PASS both paths down, 502",flush=True)
        run(CORE,"config","set","request_timeout_ms","5000")
        publish([{"version":128,"frame":"tlv","body":"selective-ack"}])
        wait(lambda: request("/v1/auth/info",token=token).get("exchange_module_versions")==[128])
        custom_lane(port, ctx, token, True)
        if core_version.get("alpha_profiles"):
            env.update(GARLIC_ROUTE_MODE="direct",GARLIC_STORAGE_URLS=origin,GARLIC_CONTROL_URLS="")
            proxy_port = free_port()
            def sign_module(seq, module):
                source = folder/"module-source.json"
                source.write_text(json.dumps(module))
                subprocess.run([sys.executable,RUST/"tools/sign_profile.py","--network",network,
                    "--sequence",str(seq),"--expires",str(int(time.time())+3600),"--frame-bytes","4096",
                    "--key",keyfile,"--module",source,folder/"record-profile.json"],check=True)
            def stream():
                conn = socket.create_connection(("127.0.0.1",proxy_port),3)
                conn.settimeout(12)
                target = f"127.0.0.1:{echo.server_address[1]}"
                conn.sendall(f"CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n".encode())
                with conn.makefile("rb",buffering=0) as head:
                    assert b" 200 " in head.readline()
                    while head.readline() not in (b"",b"\r\n"): pass
                return conn
            sign_module(2,dict(version=128,frame="tlv",body="selective-ack"))
            with process(CORE,"proxy","--bind",f"127.0.0.1:{proxy_port}"):
                wait(ready)
                with stream() as old:
                    roundtrip(old)
                    new = dict(version=129,frame="fragments",body="base")
                    publish([new])
                    wait(lambda: request("/v1/auth/info",token=token).get("exchange_module_versions")==[129])
                    sign_module(3,new)
                    with stream() as fresh:
                        roundtrip(fresh)
                        roundtrip(old)
                    # Even the legitimate publisher cannot add a system capability.
                    invalid = dict(network=network, abi=2, sequence=4, expires=int(time.time())+3600,
                                   frame_bytes=4096, module=dict(new,command="open-file"))
                    raw = json.dumps(invalid).encode()
                    sig = publisher.sign(b"Garlic Alpha record profile v1\0"+raw)
                    (folder/"record-profile.json").write_text(json.dumps(dict(
                        payload=base64.b64encode(raw).decode(),signature=base64.b64encode(sig).decode())))
                    with stream() as fresh:
                        roundtrip(fresh)
                    slot = json.loads((folder/f"client.profile-{network}.json").read_bytes())
                    assert json.loads(base64.b64decode(slot["current"]["payload"]))["sequence"] == 3
                print("Alpha module 128 -> 129 live: both streams echoed; signed unknown capability rejected")
        publish([])
        wait(lambda: "exchange_module_versions" not in request("/v1/auth/info",token=token))
        custom_lane(port, ctx, token, False)
        print("New module 128 loaded, exchanged a selective ACK, then rejected upgrades after removal")




if __name__=="__main__":
    with tempfile.TemporaryDirectory() as path:
        test(Path(path))
