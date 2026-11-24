"""Repeatable localhost tests. Faults apply to the test session only."""
import asyncio
import json
import math
from pathlib import Path
import secrets
import statistics
import tempfile
import time

import guc
from pki import issue


async def scenario(tls, name):
    tasks = set()
    async def serve(r, w):
        task = asyncio.current_task()
        tasks.add(task)
        try:
            guc.negotiated(w)
            count = 0
            while True:
                data = await asyncio.wait_for(guc.read(r), 3)
                if data is None:
                    await guc.write(w, None)
                    return
                count += 1
                if name == "delay20ms":
                    await asyncio.sleep(.020)
                if count == 8 and name in ("pause250ms", "timeout"):
                    await asyncio.sleep(.250 if name == "pause250ms" else 1)
                if name == "disconnect" and count == 8:
                    return
                await guc.write(w, data)
        except (OSError, asyncio.IncompleteReadError, TimeoutError):
            pass
        finally:
            w.close()
            tasks.discard(task)
    server = await asyncio.start_server(serve, "127.0.0.1", 0, ssl=tls[0])
    port = server.sockets[0].getsockname()[1]
    frame = secrets.token_bytes(guc.MAX_DATA)
    outcomes, latencies, rates = [], [], []
    try:
        async with server:
            for _ in range(5):
                start = time.perf_counter()
                r, w = await asyncio.wait_for(asyncio.open_connection(
                    "127.0.0.1", port, ssl=tls[1], server_hostname="127.0.0.1"), 3)
                guc.negotiated(w)
                try:
                    for seq in range(32):
                        payload = seq.to_bytes(4, "big") + frame[4:]
                        await guc.write(w, payload)
                        echoed = await asyncio.wait_for(guc.read(r), .65)
                        if echoed != payload:
                            raise ValueError("echo mismatch")
                        if seq == 0:
                            latencies.append((time.perf_counter()-start)*1000)
                    await guc.write(w, None)
                    if await asyncio.wait_for(guc.read(r), 1) is not None:
                        raise ValueError("missing FIN")
                    rates.append(32*len(frame)*8/(time.perf_counter()-start)/1e6)
                    outcomes.append("ok")
                except asyncio.IncompleteReadError:
                    outcomes.append("closed")
                except TimeoutError:
                    outcomes.append("timeout")
                finally:
                    w.close()
                    try:
                        await w.wait_closed()
                    except OSError:
                        pass
    finally:
        for t in list(tasks):
            t.cancel()
        await asyncio.gather(*list(tasks), return_exceptions=True)
    expected = {"disconnect":"closed","timeout":"timeout"}.get(name,"ok")
    if outcomes != [expected]*5:
        raise AssertionError((name,outcomes))
    return dict(profile=name, attempts=len(outcomes), outcomes=outcomes,
                payload_bytes_per_complete_run=32*len(frame),
                median_Mbps=statistics.median(rates) if rates else None,
                first_echo_p95_ms=sorted(latencies)[math.ceil(.95*len(latencies))-1])


async def main():
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        issue(d)
        tls = guc.contexts(d)
        return [await scenario(tls, name) for name in
                ("baseline","delay20ms","pause250ms","timeout","disconnect")]


if __name__ == "__main__":
    print(json.dumps(asyncio.run(main()), indent=2))
