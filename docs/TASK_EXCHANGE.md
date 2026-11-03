# Garlic v0.2 — request/response exchange prototype (Rust)

**Status:** legacy task-exchange guide. Linux release builds and local integration tests passed; see [STATUS.md](STATUS.md) for remaining requirements. This is an application-layer HTTP task-exchange prototype, **not** an OS-level network adapter, mobile app, DNS resolver, or fully transparent IP transport.

## Included

- **Garlic DNS:** persistent task queue; priorities (high/normal/low), leases, event-like job dispatch, deduplication by `task_id`, status and result lookup. The name DNS is a project name; no DNS listener is implemented.
- **Garlic Infinite Storage:** direct `/execute` API and concurrent workers that pull tasks from Garlic DNS. Can perform application-level HTTP requests to **explicitly configured named services** and return HTTP status, selected headers, and base64-encoded response bytes.
- **Garlic Internal Network:** Windows command-line client, direct service preference, fallback to coordinator queue, task completion polling, bounded deadlines. Shared protocol is Android-portable Rust.
- **Garlic Core:** shared request/response protocol and validations.

## Windows startup

Install [Rust stable](https://rustup.rs/) with MSVC tools. From the project directory run `cargo test --workspace` and `cargo check --workspace`.

Open **four** PowerShell terminals in the root directory:

**Terminal A — demonstration HTTP service:**

```powershell
python examples/upstream.py
```

**Terminal B — coordinator:**

```powershell
$env:GARLIC_STATE_FILE="garlic-state.json"
cargo run -p garlic-dns
```

**Terminal C — Storage:**

```powershell
$env:GARLIC_SERVICES="demo=http://127.0.0.1:9000"
cargo run -p garlic-infinite-storage
```

**Terminal D — Client:**

```powershell
cargo run -p garlic-internal-network -- http demo GET /hello
cargo run -p garlic-internal-network -- http demo POST /echo "Hello from Garlic"
cargo run -p garlic-internal-network -- uppercase "hello garlic"
```

You should see `http_status=200` and a JSON response for the first two commands, and `HELLO GARLIC` for the third. These commands illustrate the task-service workflow; automated Linux checks are in `integration-tests/smoke.py`.

## Test the Infinity queue

Leave A, B, C running. In terminal D choose a non-listening direct endpoint to force the queued route:

```powershell
$env:GARLIC_STORAGE_URLS="http://127.0.0.1:9999"
cargo run -p garlic-internal-network -- http demo GET /queue-test
```

Expected: `route=infinity:...` with the same HTTP response. The Storage worker polls the queue, does the HTTP request and posts the result back. To test resiliency, stop Storage, submit a task, and restart Storage before the 45-second client deadline. A coordinator restart restores queued tasks from `garlic-state.json`. Leased read-only tasks restart in Pending state; leased mutations become Failed with outcome_unknown to avoid unsafe replay.

For a slow upstream: `cargo run -p garlic-internal-network -- http demo GET /slow`. The direct request will exceed the 5-second client deadline, then attempt the queued route. The queue-side worker has a 12-second deadline. The result can still arrive; a read-only GET may have been executed twice.

## Environment variables

| Key | Component | Default |
|---|---|---|
| `GARLIC_DNS_BIND` | DNS | `127.0.0.1:8080` |
| `GARLIC_STATE_FILE` | DNS | `garlic-state.json` |
| `GARLIC_API_KEY` | All | Unset, for loopback-only development |
| `GARLIC_DNS_URL` | Storage | `http://127.0.0.1:8080` |
| `GARLIC_STORAGE_BIND` | Storage | `127.0.0.1:8090` |
| `GARLIC_STORAGE_NAME` | Storage | `storage-a` |
| `GARLIC_SERVICES` | Storage | Empty; comma-separated `name=origin` entries |
| `GARLIC_WORKERS` | Storage | `4`, clamp 1–40 |
| `GARLIC_STORAGE_URLS` | Client | `http://127.0.0.1:8090` |
| `GARLIC_CONTROL_URLS` | Client | `http://127.0.0.1:8080` |

API operations:

- DNS: `GET /health`, `POST /tasks`, `GET /tasks/{task_id}`, `POST /workers/claim`, `POST /workers/complete`.
- Storage: `GET /health`, `POST /execute`.
- For HTTP operations, `TaskRequest.exchange` contains a logical service name, method, relative path, permitted headers and a base64 request body. `TaskView.exchange` contains status, selected headers and base64 response body.

## Security and correctness limitations

- **Local development only.** Cleartext HTTP endpoints and a static API key are not enough for public deployment; use HTTPS, identity-bound authorization, and audit controls. Default listeners bind to loopback. Do not expose the servers on the public Internet as-is.
- Storage only permits preconfigured origins; it does **not** accept arbitrary destination URLs. HTTP services can be registered over HTTPS, or HTTP on local loopback for testing. Redirects are disabled.
- Request and response payloads are capped at 1 MiB. This is a bounded job-exchange API, not streaming, sockets, or bidirectional live IP traffic.
- A unique task ID deduplicates coordinator submission. **Exactly-once execution is NOT guaranteed.** A worker may time out and an expired lease may cause duplicate upstream requests. Client routes mutating methods via one coordinator by default, but that alone does not make upstream execution idempotent. Only use mutating operations against services with their own idempotency key or deduplication.
- Queue persistence is a compact full JSON snapshot; it is not transactional durable storage and can become slow for high throughput. For production replace it with a transactional database. Completed tasks are retained indefinitely. Independent coordinators have independent state; no cross-coordinator replication exists.
- Connection rotation, adaptive latency scoring, event preemption, streaming, end-to-end encryption across components, Android UI and operating-system traffic integration remain **unimplemented**.
- All test services should be locally controlled. No claim of resistance to any particular network filter is made.

## Android direction

Move client orchestration into a portable `garlic-client-core` crate, expose JNI/UniFFI and build for `aarch64-linux-android` with the Android NDK. The CLI is Windows-first and does not yet expose an Android interface.
