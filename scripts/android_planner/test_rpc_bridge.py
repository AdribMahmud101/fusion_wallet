#!/usr/bin/env python3
import fcntl,json,os,pathlib,socket,tempfile,threading,unittest
import rpc_bridge
class BridgeTests(unittest.TestCase):
 def start(self,locked=False):
  s=rpc_bridge.Server(0,locked=locked,request_id='unit'); calls=[]
  def forward(payload):
   calls.append(payload)
   return {'result':'0x14a34'}
  s.forward=forward
  t=threading.Thread(target=s.serve_forever,daemon=True); t.start()
  self.addCleanup(lambda:(s.shutdown(),s.server_close(),t.join(timeout=2)))
  return s,calls
 def request(self,s,method,params=None):
  with socket.create_connection(s.server_address) as c:
   c.sendall((json.dumps({'token':s.token,'method':method,'params':params or []})+'\n').encode())
   return json.loads(c.makefile('rb').readline())
 def test_read_only_relay_cannot_forward_a_broadcast(self):
  s,calls=self.start()
  self.assertIn('error',self.request(s,'eth_sendRawTransaction',['0x02c0']))
  self.assertEqual([],calls)
  self.assertFalse(self.request(s,'lease')['result']['held'])
 def test_batch_cannot_smuggle_a_write(self):
  s,calls=self.start()
  self.assertIn('error',self.request(s,'batch',[{'method':'eth_sendRawTransaction','params':['0x02c0']}]))
  self.assertEqual([],calls)
 def test_batch_preserves_top_level_range_error(self):
  s,calls=self.start()
  s.forward=lambda payload:{'error':{'message':'range limit','httpStatus':413}}
  reply=self.request(s,'batch',[{'method':'eth_getLogs','params':[]}])
  self.assertEqual(413,reply['error']['httpStatus']); self.assertNotIn('result',reply)
 def test_batch_preserves_results_and_does_not_admit_a_write(self):
  s,calls=self.start()
  s.forward=lambda payload:[{'jsonrpc':'2.0','id':p['id'],'result':'0x14a34'} for p in payload]
  self.assertEqual('0x14a34',self.request(s,'batch',[{'method':'eth_chainId','params':[]}])['result'][0]['result'])
 def test_upstream_batches_are_bounded_and_preserve_response_ids(self):
  s,calls=self.start(); del s.forward
  class Connection:
   payloads=[]
   def request(self,*args,**kwargs): self.payload=json.loads(kwargs['body']); self.payloads.append(self.payload)
   def getresponse(self):
    payload=self.payload
    class Response:
     status=200
     def read(self): return json.dumps([{'id':p['id'],'result':'0x14a34'} for p in payload]).encode()
    return Response()
   def close(self): pass
  connection=Connection(); s.upstream=connection
  result=s.forward([{'jsonrpc':'2.0','id':i,'method':'eth_chainId','params':[]} for i in range(24)])
  self.assertEqual([20,4],[len(p) for p in connection.payloads]); self.assertEqual(list(range(24)),[r['id'] for r in result])
 def test_lock_lease_holds_real_flock_and_stops_writes_before_release(self):
  with tempfile.TemporaryDirectory() as d:
   old=rpc_bridge.SESSION_LOCK; rpc_bridge.SESSION_LOCK=str(pathlib.Path(d)/'session.lock')
   try: s,calls=self.start(locked=True)
   finally: rpc_bridge.SESSION_LOCK=old
   with open(s.session_lock.name,'a+') as other:
    with self.assertRaises(BlockingIOError): fcntl.flock(other,fcntl.LOCK_EX|fcntl.LOCK_NB)
    self.assertTrue(self.request(s,'lease')['result']['held'])
    self.assertEqual('0x14a34',self.request(s,'eth_sendRawTransaction',['0x02c0'])['result'])
    s.disable_writes()
    self.assertFalse(self.request(s,'lease')['result']['held'])
    self.assertIn('error',self.request(s,'eth_sendRawTransaction',['0x02c0']))
    self.assertEqual(1,len(calls))
    with self.assertRaises(BlockingIOError): fcntl.flock(other,fcntl.LOCK_EX|fcntl.LOCK_NB)
 def test_inherited_descriptor_preserves_the_same_kernel_lock(self):
  with tempfile.TemporaryDirectory() as d:
   path=str(pathlib.Path(d)/'session.lock')
   with open(path,'a+') as parent:
    fcntl.flock(parent,fcntl.LOCK_EX)
    old_path=rpc_bridge.SESSION_LOCK; old_fd=os.environ.get('K17_SESSION_LOCK_FD')
    rpc_bridge.SESSION_LOCK=path; os.environ['K17_SESSION_LOCK_FD']=str(parent.fileno())
    try: s,calls=self.start(locked=True)
    finally:
     rpc_bridge.SESSION_LOCK=old_path
     if old_fd is None: os.environ.pop('K17_SESSION_LOCK_FD',None)
     else: os.environ['K17_SESSION_LOCK_FD']=old_fd
    self.assertTrue(self.request(s,'lease')['result']['held'])
    with open(path,'a+') as competitor:
     with self.assertRaises(BlockingIOError): fcntl.flock(competitor,fcntl.LOCK_EX|fcntl.LOCK_NB)
     s.shutdown(); s.server_close()
     # Closing the child relay must leave the enclosing owner sequence locked.
     with self.assertRaises(BlockingIOError): fcntl.flock(competitor,fcntl.LOCK_EX|fcntl.LOCK_NB)
 def test_receipt_watch_retries_network_failure_and_requires_canonical_block(self):
  class Fake:
   calls=0
   def forward(self,payload):
    self.calls+=1
    if self.calls==1: raise OSError('temporary network failure')
    if payload['method']=='eth_getBlockByNumber': return {'result':{'hash':'public-block'}}
    return {'result':{'transactionHash':'public-tx','status':'0x1','blockNumber':'0x1','blockHash':'public-block'}}
  fake=Fake(); result=rpc_bridge.wait_for_receipt(fake,'public-tx',pause=lambda _:None)
  self.assertEqual('0x1',result['status']); self.assertEqual(3,fake.calls)
if __name__=='__main__': unittest.main()
