#!/usr/bin/env python3
"""Loopback transport for Android hosts with unavailable platform DNS.

No planner/model code. Only forwards allowlisted live JSON-RPC to pinned HTTPS.
Standalone mode is read-only. The locked tick driver supplies a write lease.
"""
import argparse,collections,fcntl,http.client,os,json,secrets,shlex,socketserver,subprocess,threading,time,urllib.parse
RPC='https://base-sepolia-rpc.publicnode.com'
SESSION_LOCK='/root/projects/fusion_wallet/.session.lock'
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
                out={'result':{'held':self.server.locked and self.server.accept_writes,'requestId':self.server.request_id}}
            elif method=='batch':
                calls=req['params']
                if not isinstance(calls,list) or not 1<=len(calls)<=50: raise ValueError('invalid read batch')
                if any(c['method'] not in READ for c in calls): raise ValueError('write batch forbidden')
                batch=[{'jsonrpc':'2.0','id':i,'method':c['method'],'params':c['params']} for i,c in enumerate(calls)]
                reply=self.server.forward(batch)
                # Preserve top-level HTTP/RPC errors (not a batch result array),
                # including range-limit status used by K4's adaptive log paging.
                out=reply if isinstance(reply,dict) and 'error' in reply else {'result':reply}
            else:
                if method not in READ and not (self.server.locked and self.server.accept_writes and method=='eth_sendRawTransaction'):
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
        self.token=secrets.token_hex(32); self.locked=False; self.accept_writes=False; self.request_id=request_id; self.session_lock=None
        self.inherited_lock=False
        if locked:
            inherited=os.environ.get('K17_SESSION_LOCK_FD')
            if inherited is not None:
                fd=int(inherited)
                expected=os.stat(SESSION_LOCK); actual=os.fstat(fd)
                if (expected.st_dev,expected.st_ino)!=(actual.st_dev,actual.st_ino): raise ValueError('wrong inherited lock')
                self.session_lock=os.fdopen(os.dup(fd),'a+')
                self.inherited_lock=True
            else: self.session_lock=open(SESSION_LOCK,'a+')
            fcntl.flock(self.session_lock,fcntl.LOCK_EX)
            self.locked=True; self.accept_writes=True
        self.rpc_lock=threading.Lock(); self.upstream=None; self.read_window=collections.deque()
    def forward(self,payload):
        # Keep-alive and a shared rolling read budget. Batches count per item,
        # not per HTTP request. This preserves K4 batching without provider bursts.
        with self.rpc_lock:
            write=isinstance(payload,dict) and payload.get('method')=='eth_sendRawTransaction'
            if write and (not self.locked or not self.accept_writes or self.session_lock is None or self.session_lock.closed):
                raise ValueError('session flock lost')
            parts=[payload[i:i+20] for i in range(0,len(payload),20)] if isinstance(payload,list) else [payload]
            combined=[]
            for part in parts:
                count=len(part) if isinstance(part,list) else 1
                for attempt in range(2 if not write else 1):
                    while True:
                        now=time.monotonic()
                        while self.read_window and self.read_window[0][0]<=now-1: self.read_window.popleft()
                        if sum(n for _,n in self.read_window)+count<=20: break
                        time.sleep(max(.001,self.read_window[0][0]+1-now))
                    self.read_window.append((time.monotonic(),count))
                    url=urllib.parse.urlsplit(RPC)
                    if self.upstream is None: self.upstream=http.client.HTTPSConnection(url.netloc,timeout=25)
                    try:
                        self.upstream.request('POST',url.path or '/',body=json.dumps(part),headers={'Content-Type':'application/json','User-Agent':'fusion-wallet-android/1.0'})
                        response=self.upstream.getresponse(); body=response.read()
                        if response.status!=200:
                            if not write and attempt==0 and response.status in (429,502,503,504):
                                time.sleep(.25); continue
                            return {'error':{'message':'RPC HTTP range/availability error','httpStatus':response.status}}
                        value=json.loads(body)
                        break
                    except Exception:
                        self.upstream.close(); self.upstream=None
                        # Never retry a write: its durable app journal must resolve
                        # an ambiguous send by hash while the flock remains held.
                        if write or attempt: raise
                if isinstance(payload,list):
                    if not isinstance(value,list): return value
                    combined.extend(value)
                else: return value
            return combined
    def disable_writes(self):
        with self.rpc_lock: self.accept_writes=False
    def server_close(self):
        super().server_close()
        with self.rpc_lock:
            if self.upstream: self.upstream.close(); self.upstream=None
            self.locked=False
            if self.session_lock and not self.session_lock.closed:
                # A duplicated inherited FD shares the parent's lock. Explicit
                # LOCK_UN would also unlock the owner's enclosing sequence.
                if not self.inherited_lock: fcntl.flock(self.session_lock,fcntl.LOCK_UN)
                self.session_lock.close()
    def config(self): return json.dumps({'host':'127.0.0.1','port':self.server_address[1],'token':self.token,'requestId':self.request_id})
def wait_for_receipt(server,tx_hash,pause=time.sleep):
    # Read failures must not drop the session flock around an unresolved signed tx.
    while True:
        try:
            receipt=server.forward({'jsonrpc':'2.0','id':1,'method':'eth_getTransactionReceipt','params':[tx_hash]}).get('result')
            if isinstance(receipt,dict) and receipt.get('transactionHash','').lower()==tx_hash.lower() and receipt.get('status') in ('0x0','0x1'):
                block=server.forward({'jsonrpc':'2.0','id':1,'method':'eth_getBlockByNumber','params':[receipt['blockNumber'],False]}).get('result')
                if isinstance(block,dict) and block.get('hash')==receipt.get('blockHash'): return receipt
        except Exception: pass
        pause(2)
def command(local_host,serial,*args,**kwargs):
    cmd=['chroot','/proc/1/root','/system/bin/sh','-c','PATH=/system/bin "$@"', '--', *args] if local_host else ['adb','-s',serial,'shell',shlex.join(args)]
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
