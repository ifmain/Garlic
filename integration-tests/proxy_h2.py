"""Local TLS/h2 transport tests. Requires Python h2 and Caddy; no public network.
Tests entry forwarding against a mock exit, and real exit DNS against mock DoH.
Does not validate public Internet egress or Chromium fingerprints.
"""
import os, pathlib, socket, ssl, subprocess, tempfile, threading, time
from h2.connection import H2Connection
from h2.config import H2Configuration
from h2.events import RequestReceived, DataReceived, StreamEnded, ResponseReceived
ROOT=pathlib.Path(__file__).resolve().parents[1]
KEY='integration-only-key-'+('x'*32)
processes=[]
errors=[]
class Fixture:
 def __init__(self,cert,key):
  self.ctx=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);self.ctx.load_cert_chain(cert,key);self.ctx.set_alpn_protocols(['h2'])
  self.sock=socket.create_server(('127.0.0.1',18445));self.sock.settimeout(.2);self.done=False;self.primary=0;self.fallback=0
  self.thread=threading.Thread(target=self.run,daemon=True);self.thread.start()
 def run(self):
  while not self.done:
   try:s,_=self.sock.accept()
   except TimeoutError:continue
   except OSError:return
   threading.Thread(target=self.handle,args=(s,),daemon=True).start()
 def handle(self,s):
  try:
   with self.ctx.wrap_socket(s,server_side=True) as s:
    s.settimeout(15);c=H2Connection(config=H2Configuration(client_side=False,header_encoding='utf-8'));c.initiate_connection();s.sendall(c.data_to_send());streams={}
    while data:=s.recv(65536):
     for e in c.receive_data(data):
      if isinstance(e,RequestReceived):
       h=dict(e.headers);path=h[':path'];streams[e.stream_id]=[path,bytearray()]
       if path=='/v1/tunnel':
        assert h.get('authorization')=='Bearer '+KEY
        c.send_headers(e.stream_id,[(':status','200')])
      elif isinstance(e,DataReceived):
       path,b=streams[e.stream_id];c.acknowledge_received_data(e.flow_controlled_length,e.stream_id)
       if path=='/v1/tunnel':c.send_data(e.stream_id,e.data)
       else:b.extend(e.data)
      elif isinstance(e,StreamEnded):
       path,b=streams[e.stream_id]
       if path=='/v1/tunnel':c.end_stream(e.stream_id)
       elif path=='/fail':
        self.primary+=1;c.send_headers(e.stream_id,[(':status','503')],end_stream=True)
       else:
        self.fallback+=1;b[2]|=128
        c.send_headers(e.stream_id,[(':status','200'),('content-type','application/dns-message')]);c.send_data(e.stream_id,bytes(b),end_stream=True)
     s.sendall(c.data_to_send())
  except (ConnectionError,ssl.SSLError,TimeoutError):pass
  except Exception as e:errors.append(repr(e))
 def stop(self):self.done=True;self.sock.close();self.thread.join(1)
def start(cmd,env,log):
 p=subprocess.Popen(cmd,env={**os.environ,**env,'NO_PROXY':'localhost,127.0.0.1'},stdout=log,stderr=log);processes.append(p);return p
def request(port,path,body,key=KEY,cert=None,target=None):
 ctx=ssl.create_default_context(cafile=cert);ctx.set_alpn_protocols(['h2'])
 with ctx.wrap_socket(socket.create_connection(('127.0.0.1',port),timeout=5),server_hostname='localhost') as s:
  assert s.selected_alpn_protocol()=='h2'
  c=H2Connection();c.initiate_connection();c.send_headers(1,[(':method','POST'),(':scheme','https'),(':authority',f'localhost:{port}'),(':path',path),('authorization','Bearer '+key),('content-type','application/dns-message')]+([('x-garlic-target',target)] if target else []));c.send_data(1,body,end_stream=True);s.sendall(c.data_to_send());status=None;out=bytearray()
  while data:=s.recv(65536):
   for e in c.receive_data(data):
    if isinstance(e,ResponseReceived):status=int(dict(e.headers)[b':status'])
    if isinstance(e,DataReceived):out.extend(e.data);c.acknowledge_received_data(e.flow_controlled_length,e.stream_id)
    if isinstance(e,StreamEnded):return status,bytes(out)
   s.sendall(c.data_to_send())
  raise AssertionError('response incomplete')
with tempfile.TemporaryDirectory() as tmp:
 tmp=pathlib.Path(tmp);cert=str(tmp/'cert.pem');key=str(tmp/'key.pem');log=open(tmp/'test.log','w+')
 ca=str(tmp/'ca.pem');cakey=str(tmp/'ca-key.pem')
 def openssl(*args):subprocess.run(['openssl',*args],check=True,stdout=log,stderr=log)
 openssl('req','-x509','-newkey','rsa:2048','-nodes','-keyout',cakey,'-out',ca,'-days','1','-subj','/CN=Garlic Test Root','-addext','basicConstraints=critical,CA:TRUE')
 openssl('req','-newkey','rsa:2048','-nodes','-keyout',key,'-out',str(tmp/'leaf.csr'),'-subj','/CN=localhost')
 (tmp/'extensions').write_text('subjectAltName=DNS:localhost\nbasicConstraints=critical,CA:FALSE\nextendedKeyUsage=serverAuth\n')
 openssl('x509','-req','-in',str(tmp/'leaf.csr'),'-CA',ca,'-CAkey',cakey,'-CAcreateserial','-out',cert,'-days','1','-extfile',str(tmp/'extensions'))
 fixture=Fixture(cert,key)
 try:
  caddy=tmp/'Caddyfile';caddy.write_text(f'''{{
 admin off
 auto_https off
 servers {{
  protocols h1 h2
 }}
}}
https://localhost:18443 {{
 tls {cert} {key}
 @notH2 expression `{{http.request.proto}} != "HTTP/2.0"`
 handle @notH2 {{
  respond "HTTP/2 required" 505
 }}
 reverse_proxy h2c://127.0.0.1:18071 {{
  flush_interval -1
 }}
}}
https://localhost:18444 {{
 tls {cert} {key}
 @notH2 expression `{{http.request.proto}} != "HTTP/2.0"`
 handle @notH2 {{
  respond "HTTP/2 required" 505
 }}
 reverse_proxy h2c://127.0.0.1:18072
}}
''')
  common={'GARLIC_PROXY_KEY':KEY,'GARLIC_CA_CERT':ca}
  binary=str(ROOT/'target/release/garlic-proxy')
  start([binary,'server'],{**common,'GARLIC_PROXY_BIND':'127.0.0.1:18071','GARLIC_EXIT_URL':'https://localhost:18445','GARLIC_EXIT_KEY':KEY},log)
  start([binary,'server'],{**common,'GARLIC_PROXY_BIND':'127.0.0.1:18072','GARLIC_DOH_URL':'https://localhost:18445/fail','GARLIC_DOH_FALLBACK_URL':'https://localhost:18445/dns-query'},log)
  start([os.environ.get('CADDY','/workspace/toolchains/caddy/caddy'),'run','--config',str(caddy),'--adapter','caddyfile'],{},log)
  for _ in range(100):
   try:
    with socket.create_connection(('localhost',18443),timeout=.1):break
   except OSError:time.sleep(.1)
  query=b'\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03org\x00\x00\x01\x00\x01'
  assert request(18443,'/v1/dns',query,key='wrong',cert=ca)[0]==401
  assert request(18444,'/v1/tunnel',b'',cert=ca,target='127.0.0.1:443')[0]==403
  status,answer=request(18444,'/v1/dns',query,cert=ca)
  assert status==200 and answer[:2]==query[:2] and answer[2]&128
  assert fixture.primary>=1 and fixture.fallback>=1
  status,answer=request(18443,'/v1/dns',query,cert=ca);assert status==200 and answer[:2]==query[:2]
  try:request(18443,'/v1/dns',query,cert=None);raise AssertionError('untrusted certificate accepted')
  except ssl.SSLCertVerificationError:pass
  start([binary,'client'],{**common,'GARLIC_ENTRY_URL':'https://localhost:18443','GARLIC_LOCAL_BIND':'127.0.0.1:11080','GARLIC_LOCAL_DNS_BIND':'127.0.0.1:11053'},log)
  for _ in range(100):
   try:s=socket.create_connection(('127.0.0.1',11080),timeout=.1);break
   except OSError:time.sleep(.1)
  with s:
   s.settimeout(5);s.sendall(b'CONNECT example.org:443 HTTP/1.1\r\nHost: example.org:443\r\n\r\n');header=b''
   while not header.endswith(b'\r\n\r\n'):header+=s.recv(1)
   assert b'200 Connection Established' in header,header
   payload=bytes(range(256))*32;s.sendall(payload)
   # Receiving before closing upload proves bidirectionality rather than buffered request/response.
   result=b''
   while len(result)<len(payload):result+=s.recv(8192)
   assert result==payload
  with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as udp:
   udp.settimeout(5);udp.sendto(query,('127.0.0.1',11053));answer,_=udp.recvfrom(65535);assert answer[:2]==query[:2] and answer[2]&128
  ctx=ssl.create_default_context(cafile=ca);ctx.set_alpn_protocols(['http/1.1'])
  with ctx.wrap_socket(socket.create_connection(('127.0.0.1',18443),timeout=5),server_hostname='localhost') as conn:
   conn.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
   assert b'505' in conn.recv(4096).split(b'\r\n')[0]
  assert not errors,errors
  print('PASS: TLS verification, ALPN h2, authorization, entry relay, full-duplex binary tunnel (mock exit), tunneled DNS stub, exit DoH fallback (mock resolvers)')
 except Exception:
  log.flush();print('Fixture counters:',fixture.primary,fixture.fallback,'errors:',errors);print((tmp/'test.log').read_text());raise
 finally:
  for p in reversed(processes):
   if p.poll() is None:p.terminate()
  for p in processes:
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
  fixture.stop();log.close()
