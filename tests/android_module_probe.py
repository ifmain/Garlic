"""Exercise the signed release APK in its own process against the local fixture."""
import base64
import contextlib
import json
import os
from pathlib import Path
import socket
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import threading
import subprocess
import time
import zipfile

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from windows_module_reload import delayed_gateway, free_port, roundtrip, wait

ROOT = Path(__file__).resolve().parents[1]

SDK = Path(os.environ["ANDROID_HOME"])
BT = SDK/"build-tools/35.0.0"
JDK = Path(os.environ["JAVA_HOME"])/"bin"



def verify(serial, folder, port, password, request, token, publish, echo):
    adb = [SDK/"platform-tools/adb.exe","-s",serial]
    env = dict(os.environ,JAVA_HOME=str(JDK.parent),PATH=str(JDK)+os.pathsep+os.environ["PATH"])
    def run(*args):
        return subprocess.check_output(args,env=env,text=True,errors="replace",timeout=90).strip()
    publisher = Ed25519PrivateKey.generate()
    owner = request("/v1/auth/info",token=token)["authority"]["owner_id"]
    assets = folder/"assets/fixture"
    assets.mkdir(parents=True)
    (assets/"ca.pem").write_bytes((folder/"ca.pem").read_bytes())
    (assets/f"client.profile-{owner}.pub").write_bytes(publisher.public_key().public_bytes_raw())
    def offer(seq,module):
        raw = json.dumps(dict(network=owner,abi=2,expires=int(time.time())+3600,sequence=seq,
                              frame_bytes=4096,module=module)).encode()
        sig = publisher.sign(b"Garlic Alpha record profile v1\0"+raw)
        tmp = folder/"offer.tmp"
        tmp.write_text(json.dumps(dict(payload=base64.b64encode(raw).decode(),signature=base64.b64encode(sig).decode())))
        os.replace(tmp,folder/"record-profile.json")
    def activate(module):
        publish([module])
        wait(lambda: request("/v1/auth/info",token=token).get("exchange_module_versions")==[module["version"]])
    manifest = folder/"AndroidManifest.xml"
    manifest.write_text('<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="garlic.lab.moduletest"><uses-sdk android:minSdkVersion="29" android:targetSdkVersion="30"/><application/><instrumentation android:name="garlic.lab.ModuleProbe" android:targetPackage="ru.ifmain.garlic"/></manifest>')
    jar = SDK/"platforms/android-35/android.jar"
    classes = folder/"classes"
    classes.mkdir()
    run(JDK/"javac.exe","--release","8","-cp",jar,"-d",classes,ROOT/"tests/android/ModuleProbe.java")
    run(BT/"d8.bat","--lib",jar,"--min-api","29","--output",folder,*classes.rglob("*.class"))
    unsigned = folder/"unsigned.apk"
    run(BT/"aapt.exe","package","-f","-I",jar,"-M",manifest,"-A",assets.parent,"-F",unsigned)
    with zipfile.ZipFile(unsigned,"a") as z: z.write(folder/"classes.dex","classes.dex")
    probe = folder/"probe.apk"
    run(BT/"apksigner.bat","sign","--ks",Path(os.environ.get("GARLIC_TEST_KEYSTORE", ROOT/"target/android/debug.p12")),"--ks-pass","pass:android","--out",probe,unsigned)
    origin = f"https://127.0.0.1:{port}"
    proxy = free_port()
    with contextlib.ExitStack() as clean:
        for package,apk in [("ru.ifmain.garlic",ROOT/"dist/garlic-alpha.apk"),("garlic.lab.moduletest",probe)]:
            if run(*adb,"shell","pm","list","packages",package):
                raise RuntimeError(f"Existing {package} installation; use a clean test user")
            if "Success" not in run(*adb,"install",apk): raise RuntimeError("install failed")
            clean.callback(run,*adb,"uninstall",package)
        run(*adb,"reverse","--no-rebind",f"tcp:{port}",f"tcp:{port}")
        clean.callback(run,*adb,"reverse","--remove",f"tcp:{port}")
        local = int(run(*adb,"forward","tcp:0",f"tcp:{proxy}"))
        clean.callback(run,*adb,"forward","--remove",f"tcp:{local}")
        # Discovery succeeds on the direct gateway, but opening a stream hangs.
        timed_out = threading.Event()
        class DeadDirect(BaseHTTPRequestHandler):
            def do_GET(self):
                info = dict(request("/v1/auth/info",token=token),device_leases=False)
                self.send_response(200)
                self.end_headers()
                self.wfile.write(json.dumps(info).encode())
            def do_POST(self):
                if self.path == "/v1/infinity/streams":
                    timed_out.set()
                time.sleep(8)
            def log_message(self,*args):
                pass
        dead = clean.enter_context(ThreadingHTTPServer(("127.0.0.1",0),DeadDirect))
        threading.Thread(target=dead.serve_forever,daemon=True).start()
        clean.callback(dead.shutdown)
        dead_port = dead.server_port
        run(*adb,"reverse","--no-rebind",f"tcp:{dead_port}",f"tcp:{dead_port}")
        clean.callback(run,*adb,"reverse","--remove",f"tcp:{dead_port}")
        delayed_port = clean.enter_context(delayed_gateway(port,timed_out))
        run(*adb,"reverse","--no-rebind",f"tcp:{delayed_port}",f"tcp:{delayed_port}")
        clean.callback(run,*adb,"reverse","--remove",f"tcp:{delayed_port}")
        one = dict(version=128,frame="tlv",body="selective-ack")
        two = dict(version=129,frame="fragments",body="base")
        activate(one); offer(1,one)
        with (folder/"instrument.log").open("wb") as log:
            child = subprocess.Popen([*adb,"shell","am","instrument","-w","-r","-e","origin",origin,
                "-e","ingress",f"https://127.0.0.1:{delayed_port}","-e","direct",f"http://127.0.0.1:{dead_port}","-e","secret",password,"-e","bind",f"127.0.0.1:{proxy}","garlic.lab.moduletest/garlic.lab.ModuleProbe"],stdout=log,stderr=log)
            try:
                def ready():
                    output = (folder/"instrument.log").read_text(errors="replace")
                    if child.poll() is not None: raise ValueError(output)
                    return "proxy ready" in output
                wait(ready)
                @contextlib.contextmanager
                def stream(version):
                    with socket.create_connection(("127.0.0.1",local),4) as conn:
                        conn.settimeout(12)
                        target = f"127.0.0.1:{echo}"
                        conn.sendall(f"CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n".encode())
                        with conn.makefile("rb",buffering=0) as head:
                            assert b" 200 " in head.readline()
                            while head.readline() not in (b"",b"\r\n"): pass
                        roundtrip(conn)
                        assert f"exchange_open protocol={version} " in (folder/"garlic-server.log").read_text()
                        yield conn
                with stream(128) as old:
                    assert timed_out.is_set(), "direct route timeout was not exercised"
                    with socket.create_connection(("127.0.0.1",local),5) as status_socket:
                        status_socket.sendall(b"GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n")
                        with status_socket.makefile("rb") as response:
                            while response.readline() not in (b"",b"\r\n"): pass
                            assert json.load(response)["last_route"]["via"] == "through_ru"
                    print("Android APK: direct stream stalled; ingress echoed 40000 bytes",flush=True)
                    activate(two); offer(2,two)
                    with stream(129): roundtrip(old)
                    # Advertise only the existing module. An invalid replacement must
                    # preserve it. No built-in protocol can pass this transfer.
                    offer(3,dict(two,version=130,command="open-file"))
                    with stream(129): roundtrip(old)
                    print("Android APK: module 128 active; 129 active on a new stream; old stream survived; signed unknown capability rejected.",flush=True)
            finally:
                run(*adb,"shell","am","force-stop","ru.ifmain.garlic")
                child.terminate()
                child.wait(timeout=10)
