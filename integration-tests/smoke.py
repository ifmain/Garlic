import base64, json, os, pathlib, subprocess, tempfile, time, urllib.request, urllib.error, uuid
import http.server, threading
class Echo(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        body=self.rfile.read(int(self.headers['Content-Length']))
        self.send_response(418); self.send_header('Content-Length',str(len(body))); self.end_headers(); self.wfile.write(body)
    def log_message(self,*args): pass
upstream=http.server.ThreadingHTTPServer(('127.0.0.1',19000),Echo)
threading.Thread(target=upstream.serve_forever,daemon=True).start()
root=pathlib.Path(__file__).resolve().parents[1]
processes=[]
key='local-smoke-test'
def call(path, data=None, auth=True):
    req=urllib.request.Request('http://127.0.0.1:18080'+path, data=None if data is None else json.dumps(data).encode(),headers={'Content-Type':'application/json',**({'x-garlic-key':key} if auth else {})})
    with urllib.request.urlopen(req,timeout=3) as r: return json.load(r)
def start(name, **extra):
    p=subprocess.Popen([str(root/'target/release'/name)],env={**os.environ,'GARLIC_API_KEY':key,**extra},stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    processes.append(p); return p
with tempfile.TemporaryDirectory() as tmp:
 try:
    state=tmp+'/state.json'
    dns=start('garlic-dns',GARLIC_DNS_BIND='127.0.0.1:18080',GARLIC_STATE_FILE=state)
    for _ in range(50):
      try: call('/health'); break
      except OSError: time.sleep(.1)
    task={'task_id':str(uuid.uuid4()),'operation':'reverse','input':'abc'}
    assert call('/tasks',task)['status']=='pending'
    assert call('/tasks',task)['task']['task_id']==task['task_id']
    try: call('/tasks',{**task,'input':'conflict'}); raise AssertionError('conflict accepted')
    except urllib.error.HTTPError as e: assert e.code==409
    try: call('/tasks/'+task['task_id'],auth=False); raise AssertionError('unauthorized accepted')
    except urllib.error.HTTPError as e: assert e.code==401
    dns.terminate(); dns.wait()
    start('garlic-dns',GARLIC_DNS_BIND='127.0.0.1:18080',GARLIC_STATE_FILE=state)
    time.sleep(.3)
    start('garlic-infinite-storage',GARLIC_STORAGE_BIND='127.0.0.1:18090',GARLIC_DNS_URL='http://127.0.0.1:18080',GARLIC_SERVICES='demo=http://127.0.0.1:19000')
    for _ in range(100):
      result=call('/tasks/'+task['task_id'])
      if result['status']=='completed': break
      time.sleep(.1)
    assert result['result']=='cba',result
    req=urllib.request.Request('http://127.0.0.1:18090/execute',data=json.dumps(task).encode(),headers={'Content-Type':'application/json','x-garlic-key':key})
    with urllib.request.urlopen(req) as r: assert json.load(r)['result']=='cba'
    payload=bytes(range(256))*100
    http_task={'task_id':str(uuid.uuid4()),'operation':'http','exchange':{'service':'demo','method':'POST','path':'/echo','body_base64':base64.b64encode(payload).decode()}}
    call('/tasks',http_task)
    for _ in range(100):
      result=call('/tasks/'+http_task['task_id'])
      if result['status'] in ['completed','failed']: break
      time.sleep(.1)
    assert result['exchange']['status']==418,result
    assert base64.b64decode(result['exchange']['body_base64'])==payload
    print('PASS: HTTP binary roundtrip and upstream 418 status')
    print('PASS: direct, Infinity, restart persistence, deduplication, conflict, authentication')
 finally:
    upstream.shutdown()
    for p in processes:
      if p.poll() is None: p.terminate(); p.wait()
