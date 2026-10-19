#!/usr/bin/env python3
"""Loopback transport for Android hosts with unavailable platform DNS.

No planner/model code. Only forwards allowlisted live JSON-RPC to pinned HTTPS.
Standalone mode is read-only. The locked tick driver supplies a write lease.
"""
import argparse,http.client,json,secrets,shlex,socketserver,subprocess,threading,urllib.parse
RPC='https://base-sepolia-rpc.publicnode.com'
READ={'eth_chainId','eth_call','eth_getLogs','eth_getBlockByNumber','eth_getTransactionCount',
      'eth_getBalance','eth_estimateGas','eth_maxPriorityFeePerGas','eth_getTransactionReceipt','eth_getTransactionByHash','eth_getCode','eth_getStorageAt'}
class Gateway(socketserver.StreamRequestHandler):
    def handle(self):
        try:
            data=self.rfile.readline(262145)
            if len(data)>262144: raise ValueError('request too large')
            req=json.loads(data)
            if not secrets.compare_digest(req.get('token',''),self.server.token): raise ValueError('bad lease')
            method=req['method']
            if method=='lease':
                out={'result':{'held':self.server.locked,'requestId':self.server.request_id}}
            else:
                if method not in READ and not (self.server.locked and method=='eth_sendRawTransaction'):
                    raise ValueError('method unavailable without session flock')
                payload={'jsonrpc':'2.0','id':1,'method':method,'params':req['params']}
                out=self.server.forward(payload)
            self.wfile.write(json.dumps(out,separators=(',',':')).encode()+b'\n')
        except Exception:
            self.wfile.write(b'{"error":{"message":"Edge RPC transport failed"}}\n')
class Server(socketserver.ThreadingTCPServer):
    allow_reuse_address=True
    daemon_threads=True
    def __init__(self,port=8766,locked=False,request_id='read-only'):
        super().__init__(('127.0.0.1',port),Gateway)
        self.token=secrets.token_hex(32); self.locked=locked; self.request_id=request_id
        self.rpc_lock=threading.Lock(); self.upstream=None
    def forward(self,payload):
        # Reuse HTTPS connections: a fresh TLS handshake for every state word can
        # make K4's strictly pinned snapshot exceed its five-block freshness bound.
        with self.rpc_lock:
            url=urllib.parse.urlsplit(RPC)
            if self.upstream is None: self.upstream=http.client.HTTPSConnection(url.netloc,timeout=25)
            try:
                self.upstream.request('POST',url.path or '/',body=json.dumps(payload),headers={'Content-Type':'application/json','User-Agent':'fusion-wallet-android/1.0'})
                response=self.upstream.getresponse(); body=response.read()
                if response.status!=200:
                    return {'error':{'message':'RPC HTTP range/availability error','httpStatus':response.status}}
                return json.loads(body)
            except Exception:
                self.upstream.close(); self.upstream=None
                raise
    def config(self): return json.dumps({'host':'127.0.0.1','port':self.server_address[1],'token':self.token,'requestId':self.request_id})
def device(local_host=False,serial='192.168.0.105:5555'):
    return ['chroot','/proc/1/root','/system/bin/'] if local_host else ['adb','-s',serial]
def command(local_host,serial,*args,**kwargs):
    cmd=['chroot','/proc/1/root','/system/bin/'+args[0],*args[1:]] if local_host else ['adb','-s',serial,'shell',shlex.join(args)]
    return subprocess.run(cmd,check=True,**kwargs)
def provision(server,local_host,serial):
    if not local_host:
        subprocess.run(['adb','-s',serial,'reverse','tcp:'+str(server.server_address[1]),'tcp:'+str(server.server_address[1])],check=True)
    command(local_host,serial,'run-as','app.fusion.wallet','mkdir','-p','files/agent',stdout=subprocess.DEVNULL)
    command(local_host,serial,'run-as','app.fusion.wallet','sh','-c','cat > files/agent/edge_rpc.json',input=server.config().encode())
if __name__=='__main__':
    ap=argparse.ArgumentParser(); ap.add_argument('--local-host',action='store_true'); ap.add_argument('--serial',default='192.168.0.105:5555'); ap.add_argument('--port',type=int,default=8766)
    args=ap.parse_args(); server=Server(args.port); provision(server,args.local_host,args.serial)
    print('Read-only live RPC relay ready on loopback port',server.server_address[1],flush=True)
    server.serve_forever()
