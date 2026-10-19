#!/usr/bin/env python3
"""Replace only llm.cpp.o in the supplied Android build, keeping its other objects.

The original source and libraries are read-only. The bounds patch and build inputs
are recorded; this is a patched fork build, never claimed byte-identical to M1.
"""
import difflib,hashlib,json,os,pathlib,shlex,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2]
SRC=pathlib.Path(os.environ.get('MNN_SOURCE','/root/projects/victor_fusion/ahem/third_party/MNN'))
BUILD=SRC/'build_a64_qnn'
OUT=ROOT/'src/fusion-planner/android/target/mnn-patched'
OUT.mkdir(parents=True,exist_ok=True)
source=SRC/'transformers/llm/engine/src/llm.cpp'
s=source.read_text()
old='''    int cols = info->dim[info->size - 1];
    int rows = 1;
    for (int i = 0; i < info->size - 1; ++i) {
        rows *= info->dim[i];
    }'''
new='''    // Variable::Info::size counts ELEMENTS, not dimensions. This is the
    // dimension vector's rank; never index dim using the element count.
    if (info->dim.empty()) return run_fallback_gate();
    const size_t rank = info->dim.size();
    int cols = info->dim.back();
    int rows = 1;
    for (size_t i = 0; i + 1 < rank; ++i) {
        if (info->dim[i] <= 0 || rows > 2147483647 / info->dim[i])
            return run_fallback_gate();
        rows *= info->dim[i];
    }'''
assert s.count(old)==1,'Pinned MNN source changed; review the dimension patch'
patched=s.replace(old,new)
(OUT/'llm.cpp').write_text(patched)
patch=''.join(difflib.unified_diff(s.splitlines(True),patched.splitlines(True),fromfile='a/transformers/llm/engine/src/llm.cpp',tofile='b/transformers/llm/engine/src/llm.cpp'))
(ROOT/'scripts/android_planner/mnn_dimension_bounds.patch').write_text(patch)
flags={}
for line in (BUILD/'CMakeFiles/llm.dir/flags.make').read_text().splitlines():
 if ' = 'in line:
  k,v=line.split(' = ',1); flags[k]=shlex.split(v.replace('/home/adrib/victor_fusion/ahem/third_party/MNN',str(SRC)))
ndk=pathlib.Path(os.environ.get('ANDROID_NDK_ROOT','/opt/android-sdk/ndk/27.0.12077973'))
tool=ndk/'toolchains/llvm/prebuilt/linux-x86_64'
common=['clang++','--target=aarch64-linux-android34','--sysroot='+str(tool/'sysroot'),'-resource-dir',str(tool/'lib/clang/18'),'-fuse-ld=lld']
compile=common+['-I'+str(source.parent)]+flags['CXX_DEFINES']+flags['CXX_INCLUDES']+['-I'+str(tool/'sysroot/usr/include/c++/v1')]+flags['CXX_FLAGS']+['-c',str(OUT/'llm.cpp'),'-o',str(OUT/'llm.cpp.o')]
subprocess.run(compile,check=True)
original=shlex.split((BUILD/'CMakeFiles/llm.dir/link.txt').read_text())
objects=[str(OUT/'llm.cpp.o') if p.endswith('/llm.cpp.o') else str(BUILD/p) for p in original if p.endswith('.o')]
libs=[str(BUILD/p) for p in original if p.endswith('.so') and not p.startswith('-') and not p.endswith('/libllm.so')]
link=common+['-nostdlib++','-shared','-fPIC','-Wl,--build-id=sha1','-Wl,--no-undefined','-Wl,--gc-sections','-Wl,-soname,libllm.so','-o',str(OUT/'libllm.so')]+objects+libs+['-ldl','-pthread','-llog','-lm','-landroid',str(tool/'sysroot/usr/lib/aarch64-linux-android/libc++_static.a'),str(tool/'sysroot/usr/lib/aarch64-linux-android/libc++abi.a')]
subprocess.run(link,check=True)
sha=lambda p:hashlib.file_digest(pathlib.Path(p).open('rb'),'sha256').hexdigest()
record={'sourceCommit':subprocess.check_output(['git','-C',str(SRC),'rev-parse','HEAD'],text=True).strip(),
 'originalSourceSha256':sha(source),'patchedSourceSha256':sha(OUT/'llm.cpp'),'patchSha256':hashlib.sha256(patch.encode()).hexdigest(),
 'originalLibllmSha256':sha(BUILD/'OFF/arm64-v8a/libllm.so'),'patchedLibllmUnstrippedSha256':sha(OUT/'libllm.so'),
 'compile':compile,'link':link,'note':'Only llm.cpp rebuilt with the bounds fix; other Android MNN objects/libraries unchanged. NDK27 libc++ relink, target API34.'}
(ROOT/'evidence/app/agent/mnn_patch_build.json').write_text(json.dumps(record,indent=2)+'\n')
print('Patched libllm built:',OUT/'libllm.so')
