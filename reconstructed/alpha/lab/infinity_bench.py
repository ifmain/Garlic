"""Benchmark the real Infinity HTTP API and local executor using disposable identities."""
import argparse
import http.client
from contextlib import ExitStack
from concurrent.futures import ThreadPoolExecutor
import json
import math
import os
from pathlib import Path
import secrets
import socket
import ssl
import subprocess
import tempfile
import time
from urllib.request import build_opener, HTTPSHandler, ProxyHandler, Request
import uuid

from pki import issue
from link import link


def trial(binary_dir, workers, clients, count, profile="baseline", exchange_check=False):
    with tempfile.TemporaryDirectory() as tmp, ExitStack() as stack:
        d = Path(tmp)
        issue(d)
        with socket.socket() as s:
            s.bind(("127.0.0.1", 0))
            port = s.getsockname()[1]
        addr = f"127.0.0.1:{port}"
        url = "https://" + addr
        front, blackhole = stack.enter_context(link(port, profile))
        api_url = "https://127.0.0.1:" + str(front)
        env = {k: v for k, v in os.environ.items() if not k.startswith("GARLIC_")}
        env.update(GARLIC_AUTH_FILE=str(d/"accounts.json"), GARLIC_INFRA_KEYS_FILE=str(d/"keys.json"),
                   GARLIC_STORAGE_BIND=addr, GARLIC_DNS_URL=url, GARLIC_PUBLIC_URL=url,
                   GARLIC_DIRECT_ONLY="1", GARLIC_WORKERS=str(workers), GARLIC_NODE_LABEL="lab",
                   GARLIC_NODE_IDENTITY_FILE=str(d/"identity.json"), GARLIC_CA_CERT=str(d/"ca.pem"),
                   GARLIC_TLS_CERT=str(d/"server.pem"), GARLIC_TLS_KEY=str(d/"server.key"))
        password = secrets.token_urlsafe(24)
        if exchange_check:
            env.update(GARLIC_DEV_MODE="1",GARLIC_DEV_ALLOW_PRIVATE_PROXY_TARGETS="1")
        def auth(*args, stdin=None):
            subprocess.run([binary_dir/"garlic-auth", *args], input=stdin, cwd=d, env=env,
                           check=True, text=True, stdout=subprocess.DEVNULL)
        auth("keys", "init", d/"keys.json", "lab")
        auth("users", "init", d/"accounts.json")
        auth("users", "add", d/"accounts.json", "benchmark", "--password-stdin", stdin=password+"\n")
        token = None
        tls = ssl.create_default_context(cafile=str(d/"ca.pem"))
        def api(path, body=None):
            headers = {"Content-Type":"application/json"}
            if token:
                headers["Authorization"] = "Bearer " + token
            req = Request(api_url+path, data=None if body is None else json.dumps(body).encode(), headers=headers)
            opener = build_opener(ProxyHandler({}), HTTPSHandler(context=tls))
            with opener.open(req, timeout=5) as response:
                return json.load(response)
        def task(_):
            started = time.perf_counter()
            ident = str(uuid.uuid4())
            api("/v1/infinity/tasks", {"operation":"echo", "input":ident, "task_id":ident})
            while time.perf_counter() - started < 30:
                result = api("/v1/infinity/tasks/"+ident)
                if result["status"] == "completed":
                    if result["result"] != ident:
                        raise ValueError("incorrect echo")
                    return time.perf_counter() - started
                if result["status"] == "failed":
                    raise RuntimeError("executor failed")
                time.sleep(.025)
            raise TimeoutError("Infinity job did not finish within 30s")
        with (d/"service.log").open("wb") as log:
            proc = subprocess.Popen([binary_dir/"garlic-server", "--role", "exit"],
                                    env=env, cwd=d, stdout=log, stderr=log)
            try:
                for _ in range(100):
                    if proc.poll() is not None:
                        raise RuntimeError((d/"service.log").read_text(errors="replace")[-500:])
                    try:
                        api("/health")
                        break
                    except OSError:
                        time.sleep(.1)
                token = api("/v1/auth/login", {"username":"benchmark","password":password})["token"]
                if exchange_check:
                    from exchanges import verify
                    return verify(binary_dir/"garlic-internal-network",d,env,password,url)
                task(None)
                if profile == "reconnect":
                    blackhole()
                    test = build_opener(ProxyHandler({}), HTTPSHandler(context=tls))
                    try:
                        test.open(api_url+"/health", timeout=.25).close()
                    except OSError:
                        pass
                    else:
                        raise AssertionError("blackout had no effect")
                    # Discard the first reply after the server has accepted the POST.
                    # Retrying the same ID must return the existing task (HTTP 200).
                    nonce = str(uuid.uuid4())
                    body = json.dumps({"operation":"echo","input":nonce,"task_id":nonce})
                    for expected in [202,200]:
                        conn = http.client.HTTPSConnection("127.0.0.1",front,context=tls,timeout=5)
                        try:
                            conn.request("POST","/v1/infinity/tasks",body,
                                         {"Authorization":"Bearer "+token,"Content-Type":"application/json"})
                            reply=conn.getresponse()
                            if reply.status!=expected:
                                raise AssertionError((expected,reply.status))
                        finally:
                            conn.close()
                start = time.perf_counter()
                with ThreadPoolExecutor(clients) as pool:
                    lat = sorted(pool.map(task, range(count)))
                duration = time.perf_counter()-start
                return dict(profile=profile, workers=workers, concurrency=clients, completed=len(lat), seconds=duration,
                            jobs_per_second=count/duration, p50_ms=1000*lat[math.ceil(.5*count)-1],
                            p95_ms=1000*lat[math.ceil(.95*count)-1])
            finally:
                proc.terminate()
                proc.wait(timeout=10)


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("binary_dir", type=Path)
    p.add_argument("--exchanges", action="store_true")
    p.add_argument("--count", type=int, default=24, choices=range(1, 1025), metavar="1..1024")
    a = p.parse_args()
    if a.exchanges:
        print(json.dumps(trial(a.binary_dir.resolve(),1,1,1,exchange_check=True),indent=2))
        raise SystemExit()
    cases = [(1,1,"baseline"), (1,8,"baseline"), (4,8,"baseline"), (8,16,"baseline"),
             (4,8,"delayed"), (4,8,"rate128KiB"), (4,8,"reconnect")]
    print(json.dumps([trial(a.binary_dir.resolve(), w, c, a.count, profile)
                      for w, c, profile in cases], indent=2))
