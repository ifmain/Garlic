"""Run a signed network GPF through the native client against loopback-only fixtures.

Usage: python plugin_failover.py CORE GPF [--adb ADB --serial EMULATOR]
The Android core must already be on the test emulator. Requires cryptography and paramiko.
"""
import argparse
import base64
import datetime
import hashlib
import io
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
import tempfile
import threading
import time
import uuid

import paramiko
from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, rsa
from cryptography.x509.oid import NameOID


def exact(sock, size):
    data = bytearray()
    while len(data) < size:
        part = sock.recv(size - len(data))
        if not part:
            raise EOFError()
        data.extend(part)
    return bytes(data)


def until(sock, suffix):
    data = bytearray()
    while not data.endswith(suffix):
        data.extend(exact(sock, 1))
        if len(data) > 8192:
            raise ValueError("header limit")
    return bytes(data)


def certificates(root):
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, "Loopback test")])
    now = datetime.datetime.now(datetime.timezone.utc)
    def builder(subject):
        return (x509.CertificateBuilder().subject_name(subject).issuer_name(name)
                .serial_number(x509.random_serial_number()).not_valid_before(now-datetime.timedelta(days=1))
                .not_valid_after(now+datetime.timedelta(days=1)))
    ca = builder(name).public_key(key.public_key()).add_extension(x509.BasicConstraints(ca=True,path_length=None),True).sign(key,hashes.SHA256())
    leaf = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    cert = (builder(x509.Name([x509.NameAttribute(NameOID.COMMON_NAME,"localhost")])).public_key(leaf.public_key())
            .add_extension(x509.SubjectAlternativeName([x509.IPAddress(ipaddress.ip_address("127.0.0.1"))]),False)
            .sign(key,hashes.SHA256()))
    (root/"cert.pem").write_bytes(cert.public_bytes(serialization.Encoding.PEM))
    (root/"key.pem").write_bytes(leaf.private_bytes(serialization.Encoding.PEM,serialization.PrivateFormat.PKCS8,serialization.NoEncryption()))
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(root/"cert.pem",root/"key.pem")
    return ctx, base64.b64encode(ca.public_bytes(serialization.Encoding.DER)).decode()


class Login(paramiko.ServerInterface):
    def __init__(self, password):
        self.password = password
        self.dest = None

    def check_auth_password(self, username, password):
        return paramiko.AUTH_SUCCESSFUL if (username,password)==("fixture",self.password) else paramiko.AUTH_FAILED

    def check_channel_direct_tcpip_request(self, chanid, origin, destination):
        self.dest = destination
        return paramiko.OPEN_SUCCEEDED if destination in (("fixture.invalid",443),("www.gstatic.com",443)) else paramiko.OPEN_FAILED_ADMINISTRATIVELY_PROHIBITED


def respond(sock, dest, marker):
    if dest == ("fixture.invalid",443):
        assert until(sock,b"\r\n\r\n").startswith(b"GET /check ")
        sock.sendall(b"HTTP/1.1 200 OK\r\nContent-Length: "+str(len(marker)).encode()+b"\r\n\r\n"+marker)
    elif dest == ("www.gstatic.com",443):
        with socket.create_connection(dest,timeout=5) as remote:
            while True:
                ready=select.select([sock,remote],[],[],10)[0]
                if not ready:return
                for src in ready:
                    chunk=src.recv(16384)
                    if not chunk:return
                    (sock if src==remote else remote).sendall(chunk)
    else:
        raise ValueError("destination outside fixture")


def handler(kind, password, ctx, sshkey, marker):
    class Fixture(socketserver.BaseRequestHandler):
        def handle(self):
            try:self.serve()
            except (EOFError, OSError, paramiko.SSHException):pass
        def serve(self):
            sock=self.request
            sock.settimeout(10)
            if kind=="ssh":
                with paramiko.Transport(sock) as t:
                    t.add_server_key(sshkey)
                    auth=Login(password);t.start_server(server=auth)
                    ch=t.accept(9)
                    if ch:
                        with ch:respond(ch,auth.dest,marker)
                return
            if kind=="https":sock=ctx.wrap_socket(sock,server_side=True)
            if kind in ("http","https"):
                head=until(sock,b"\r\n\r\n")
                assert b"Basic "+base64.b64encode(("fixture:"+password).encode()) in head
                host,port=head.split()[1].decode().rsplit(":",1)
                sock.sendall(b"HTTP/1.1 200 OK\r\n\r\n")
            elif kind=="socks4":
                assert exact(sock,2)==bytes([4,1]);port=struct.unpack(">H",exact(sock,2))[0]
                assert exact(sock,4)==bytes([0,0,0,1]) and until(sock,b"\0")==b"fixture\0"
                host=until(sock,b"\0")[:-1].decode();sock.sendall(bytes([0,90])+bytes(6))
            else:
                assert exact(sock,3)==bytes([5,1,2]);sock.sendall(bytes([5,2]))
                assert exact(sock,1)==b"\x01" and exact(sock,exact(sock,1)[0])==b"fixture"
                assert exact(sock,exact(sock,1)[0]).decode()==password;sock.sendall(bytes([1,0]))
                assert exact(sock,4)==bytes([5,1,0,3]);host=exact(sock,exact(sock,1)[0]).decode();port=struct.unpack(">H",exact(sock,2))[0]
                sock.sendall(bytes([5,0,0,1])+bytes(6))
            try:respond(sock,(host,int(port)),marker)
            finally:sock.close()
    return Fixture


def write(path, obj):
    path.write_text(json.dumps(obj),encoding="utf8")


def check(args, root):
    env=dict(os.environ,GARLIC_CLIENT_CONFIG=str(root/"client.json"))
    def adb(*argv,**opts):
        return subprocess.run([args.adb,"-s",args.serial,*argv],check=True,capture_output=True,**opts)
    android=bool(args.adb)
    directory="/data/local/tmp/garlic-dev-"+uuid.uuid4().hex
    def command(*argv):
        if android:return [args.adb,"-s",args.serial,"shell","env","GARLIC_CLIENT_CONFIG="+directory+"/client.json",args.core,*argv]
        return [args.core,*argv]
    if android:adb("shell","mkdir","-p",directory)
    envelope=json.loads(Path(args.gpf).read_bytes())
    raw=base64.b64decode(envelope["payload"]);author=base64.b64decode(envelope["author"])
    rsa.RSAPublicNumbers(65537,int.from_bytes(author,"big")).public_key().verify(base64.b64decode(envelope["signature"]),raw,padding.PKCS1v15(),hashes.SHA256())
    manifest=json.loads(raw);manifest["publisher"]=hashlib.sha256(author).hexdigest().upper()
    slot=root/"plugins-v3"/("s-"+manifest["id"]);slot.mkdir(parents=True)
    write(slot/"client.gpf-module",manifest)
    # A valid synthetic Garlic session points at a closed loopback port.
    dead="https://127.0.0.1:1";uid=str(uuid.uuid4())
    write(root/"client.sessions.json",{"version":1,"entries":{dead:{"token":"fixture","expires_at":4102444800,
        "principal":{"kind":"user","user_id":uid,"username":"fixture"},"authority":{"owner_id":uid,"epoch":1}}}})
    for field,value in [("storage_urls",dead),("control_urls",dead)]:
        subprocess.run(command("config","set",field,value),env=env,check=True,stdout=subprocess.DEVNULL)
    ctx,ca=certificates(root)
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    sshkey=paramiko.Ed25519Key.from_private_key(io.StringIO(Ed25519PrivateKey.generate().private_bytes(
        serialization.Encoding.PEM,serialization.PrivateFormat.OpenSSH,serialization.NoEncryption()).decode()))
    class Server(socketserver.ThreadingTCPServer):
        daemon_threads=True
    front=0
    with socket.socket() as tmp:
        tmp.bind(("127.0.0.1",0));front=tmp.getsockname()[1]
    if android:adb("forward",f"tcp:{front}",f"tcp:{front}")
    try:
        for kind in ["http","https","socks4","socks5","ssh"]:
            password=secrets.token_hex(8);marker=secrets.token_hex(18).encode()
            with Server(("127.0.0.1",0),handler(kind,password,ctx,sshkey,marker)) as server:
                port=server.server_address[1];address=f"{kind}://127.0.0.1:{port}"
                threading.Thread(target=server.serve_forever,daemon=True).start()
                if android:adb("reverse",f"tcp:{port}",f"tcp:{port}")
                row={"kind":kind,"address":address}
                data={"format":"garlic-plugin-data/1","id":manifest["id"],"enabled":True,"interval_seconds":5,"ingress_penalty_ms":0,"connections":[{"kind":"http","address":"http://127.0.0.1:1"},row]}
                write(slot/"client.installed-gpf",data)
                login={"user":"fixture","password":password,"pin":ca if kind=="https" else sshkey.fingerprint if kind=="ssh" else ""}
                def sync():
                    write(slot/"client.connector-login",{address:login})
                    if android:
                        adb("push",str(root/"client.sessions.json"),directory+"/client.sessions.json")
                        adb("push",str(root/"plugins-v3"),directory+"/")
                sync()
                proc=subprocess.Popen(command("proxy","--bind",f"127.0.0.1:{front}"),env=env,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
                try:
                    deadline=time.monotonic()+25
                    while True:
                        if proc.poll() is not None:raise RuntimeError(proc.stderr.read().decode()[-400:])
                        try:
                            with socket.create_connection(("127.0.0.1",front),timeout=1) as s:
                                s.sendall(b"CONNECT fixture.invalid:443 HTTP/1.1\r\nHost: fixture.invalid\r\n\r\n")
                                s.settimeout(12)
                                reply=until(s,b"\r\n\r\n")
                                if not reply.startswith(b"HTTP/1.1 200"):
                                    diag=subprocess.run(command("plugin-test",manifest["id"],kind,address),env=env,capture_output=True,timeout=12)
                                    raise AssertionError((kind,diag.stderr.decode(errors="replace"),reply))
                                s.sendall(b"GET /check HTTP/1.1\r\nHost: fixture.invalid\r\n\r\n")
                                until(s,b"\r\n\r\n");assert exact(s,len(marker))==marker
                            break
                        except (EOFError,ConnectionError):
                            if time.monotonic()>deadline:raise
                            time.sleep(.15)
                    if android:
                        logs=adb("shell","cat",directory+"/logs/*.log").stdout.decode()
                    else:
                        logs="".join(p.read_text() for p in (root/"logs").glob("*.log"))
                    for event in ("plugin_attempt","plugin_connected"):
                        assert any(event in line and f"protocol={kind} " in line
                                   for line in logs.splitlines()), (kind,event)
                    assert password not in logs and "fixture.invalid" not in logs
                    cmd=command("plugin-test",manifest["id"],kind,address)
                    ok=subprocess.run(cmd,env=env,capture_output=True,timeout=20)
                    print(kind,"fallback PASS; HTTPS probe",ok.returncode,ok.stderr.decode(errors="replace")[:120],flush=True)
                    if kind in ["https","ssh"]:
                        login["pin"]="SHA256:incorrect" if kind=="ssh" else ""
                        sync();bad=subprocess.run(cmd,env=env,capture_output=True,timeout=20)
                        assert bad.returncode==1 and bad.stderr.strip(),("unexpected rejection",kind,bad.returncode,bad.stderr)
                    assert ok.returncode==0
                finally:
                    subprocess.run(command("stop"),env=env,capture_output=True,timeout=10)
                    try:exit_code=proc.wait(10)
                    except subprocess.TimeoutExpired:proc.kill();proc.wait();raise
                    assert exit_code==0,(kind,exit_code,proc.stderr.read().decode(errors="replace")[-1200:])
                    if android:adb("reverse","--remove",f"tcp:{port}")
                    server.shutdown()
    finally:
        if android:
            adb("forward","--remove",f"tcp:{front}")
            adb("shell","rm","-r",directory)


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("core");parser.add_argument("gpf");parser.add_argument("--adb");parser.add_argument("--serial")
    with tempfile.TemporaryDirectory() as temp:check(parser.parse_args(),Path(temp))
