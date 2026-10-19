#!/usr/bin/env python3
"""Debug-only SESSION key import, stdin-equivalent local socket transport.

No key in adb arguments, Intent extras, plaintext files, clipboard or output.
The activity lives in :signer and accepts only shell/root peers while open.
"""
import argparse,pathlib,re,socket,subprocess,time
from rpc_bridge import command
ap=argparse.ArgumentParser(); ap.add_argument('--local-host',action='store_true'); ap.add_argument('--serial',default='192.168.0.105:5555'); args=ap.parse_args()
# The shell test may return nonzero; use a captured command without exception details.
cmd=['chroot','/proc/1/root','/system/bin/run-as','app.fusion.wallet','test','-f','no_backup/agent/session.enc'] if args.local_host else ['adb','-s',args.serial,'shell','run-as app.fusion.wallet test -f no_backup/agent/session.enc']
if subprocess.run(cmd,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode==0:
 print('Session envelope already imported; no key read.'); raise SystemExit(0)
key=None
try:
 for line in pathlib.Path('/root/projects/fusion_wallet/.env.local').open():
  match=re.match(r'^\s*(?:export\s+)?SESSION_PRIVATE_KEY\s*=\s*([\"\']?)(0x[0-9a-fA-F]{64}|[0-9a-fA-F]{64})\1\s*(?:#.*)?$',line)
  if match: key=bytearray.fromhex(match.group(2).removeprefix('0x')); break
 if key is None: raise SystemExit('SESSION_PRIVATE_KEY is missing or invalid; no key was sent.')
 command(args.local_host,args.serial,'am','start','-n','app.fusion.wallet/.agent.SessionImportActivity',stdout=subprocess.DEVNULL)
 if not args.local_host:
  port=int(subprocess.check_output(['adb','-s',args.serial,'forward','tcp:0','localabstract:fusion.session.import.v1'],text=True).strip())
 for _ in range(100):
  try:
   if args.local_host:
    client=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM); client.settimeout(10); client.connect('\0fusion.session.import.v1')
   else: client=socket.create_connection(('127.0.0.1',port),timeout=10)
   break
  except OSError:
   client.close(); time.sleep(.1)
 else: raise SystemExit('Secure import activity socket unavailable; no key was sent.')
 with client:
  client.sendall(key)
  if client.makefile('rb').readline(32)!=b'IMPORTED\n': raise SystemExit('Session import not acknowledged; inspect secure import screen.')
 print('Session imported into :signer and wrapped by Android Keystore.')
finally:
 if key is not None:
  for i in range(len(key)): key[i]=0
 if not args.local_host and 'port'in globals():
  subprocess.run(['adb','-s',args.serial,'forward','--remove','tcp:'+str(port)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
