#!/usr/bin/env python3
"""Reproduce host evaluation; never promotes missing/failed executions to PASS."""
import os,json,pathlib,subprocess,hashlib,time,statistics
ROOT=pathlib.Path(__file__).resolve().parents[2]
CRATE=ROOT/'src/fusion-planner'
E=ROOT/'evidence/planner'
BIN=CRATE/'target/release/planner-host'
def sha(p):
 h=hashlib.sha256()
 with open(p,'rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
def valid_raw(raw,labels):
 try:
  pairs=json.loads(raw,object_pairs_hook=lambda p:p)
  return len(pairs)==1 and pairs[0][0]=='choice' and pairs[0][1] in labels
 except (ValueError,TypeError,IndexError,KeyError):return False

def command(args,log):
 start=time.time()
 with open(E/log,'w') as f:
  p=subprocess.run(args,cwd=CRATE,stdout=f,stderr=subprocess.STDOUT)
 return dict(command=list(map(str,args)),exitCode=p.returncode,wallSeconds=time.time()-start,log=log)
def evaluate(mode,corpus,name):
 output=E/(name+'.jsonl')
 cmd=command([str(BIN),'--eval',str(corpus),'--mode',mode,'--out',str(output)],name+'.log')
 rows=[json.loads(x) for x in output.read_text().splitlines()] if output.exists() else []
 return cmd,rows
results={'planner':{'kind':'mnn+ahem-g'},'host':'linux-host','gitCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'gitDirty':bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT)),'tests':{},'limitations':['Native create overrides temperature to 0.3/top_k 40; both arms use same runtime.','Plain baseline uses the same fork with callback and skeleton disabled, not a stock upstream binary.','Callback budget cannot interrupt native prefill; timeout marks session FAILED after return.','Phone cross-build NEXT: NDK/cargo-ndk unavailable.']}
model=os.environ.get('FUSION_MODEL_DIR')
libs=pathlib.Path(os.environ.get('MNN_LIB_DIR',str(CRATE/'target/mnn-host')))
results['hashes']={}
if model:
 for n in ['config.json','llm_config.json','llm.mnn','llm.mnn.weight','tokenizer.mtok']:results['hashes'][n]=sha(pathlib.Path(model)/n)
for n in ['libllm.so','libMNN.so']:
 if (libs/n).exists():results['hashes'][n]=sha(libs/n)
results['originalSuppliedLibraries']={n:sha(pathlib.Path('/root/projects/victor_fusion/ahem/third_party/MNN/build')/n) for n in ['libllm.so','libMNN.so']}
results['nativeLibraryDir']=str(libs)
results['hostArchitecture']=__import__('platform').machine()
results['corpusSha256']=sha(E/'adversarial_200.jsonl')
results['modelMetadata']={'name':'LFM2.5-230M-MNN','source':'local model card','declaredLicense':'apache-2.0','revision':'not independently verified'}
results['latencyMethod']='Sequential full request timings; plain may include one validation retry. Cold load excluded; no warmup discarded.'
fork=pathlib.Path('/root/projects/victor_fusion/ahem/third_party/MNN')
results['forkCommit']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=fork,text=True).strip()
results['forkWorkingDiffSha256']=hashlib.sha256(subprocess.check_output(['git','diff','HEAD'],cwd=fork)).hexdigest()
if model:
 for mode,id in [('ahem-g','AHEM-G-01'),('plain','AHEM-G-02')]:
  cmd,rows=evaluate(mode,E/'adversarial_200.jsonl',mode+'_200')
  illegal=sum(r.get('label') not in ['A','B','H'] or (not r.get('rawValid') and not (r.get('label')=='H' and r.get('ahem',{}).get('g',{}).get('aborted'))) for r in rows)
  first_invalid=sum(not valid_raw((r.get('rawAttempts') or [r.get('raw','')])[0],['A','B','H']) for r in rows)
  results['tests'][id]={'status':'PASS' if cmd['exitCode']==0 and len(rows)==200 and (mode=='plain' or illegal==0) else 'FAIL','count':len(rows),'illegalOutputs':illegal,'invalidFirstRawOutputs':first_invalid,'invalidFinalRawOutputs':sum(not r.get('rawValid',False) for r in rows),'aborts':sum(r.get('ahem',{}).get('g',{}).get('aborted',False) for r in rows),'vetoes':sum(r.get('ahem',{}).get('g',{}).get('vetoes',0) for r in rows),'execution':cmd}
 corpus=E/'latency_20.jsonl'
 with corpus.open('w') as f:
  for i in range(20):f.write(json.dumps(dict(prompt='Choose the best available option. A is available; H means hold.',labels=['A','H'],candidateSetDigest=f'latency-{i}'))+'\n')
 latency={}
 for mode in ['ahem-g','plain']:
  cmd,rows=evaluate(mode,corpus,'latency_'+mode)
  times=sorted(r['timings']['totalMs'] for r in rows)
  latency[mode]={'count':len(times),'p50Ms':statistics.median(times) if times else None,'p95Ms':times[min(len(times)-1,__import__('math').ceil(len(times)*.95)-1)] if times else None,'execution':cmd}
 results['tests']['AHEM-G-06']={'status':'PASS' if all(x['count']==20 and x['execution']['exitCode']==0 for x in latency.values()) else 'FAIL','modes':latency}
 cmd=command([str(BIN),'--selftest'],'selftest.log')
 vals=[]
 for l in (E/'selftest.log').read_text().splitlines():
  try:vals.append(json.loads(l))
  except ValueError:pass
 details=next((x for x in vals if 'details' in x),{})
 for id in ['AHEM-G-03','AHEM-G-04','AHEM-G-05']:results['tests'][id]={'status':'PASS' if details.get('details',{}).get(id) is True else 'FAIL','execution':cmd}
 results['selftest']=details
else:
 for i in range(1,7):results['tests'][f'AHEM-G-0{i}']={'status':'NOT_RUN','reason':'FUSION_MODEL_DIR unset'}
results['tests']['AHEM-G-07']={'status':'NOT_RUN','reason':'Phone cross-build out of scope; no NDK/cargo-ndk. NEXT: arm64 Android build and ten device runs.'}
(E/'host_results.json').write_text(json.dumps(results,indent=2)+'\n')
raise SystemExit(0 if all(x['status']=='PASS' for k,x in results['tests'].items() if k!='AHEM-G-07') else 1)
