#!/bin/sh
set -eu
cd "$(dirname "$0")/../../src/fusion-planner"
cmake -S /root/projects/victor_fusion/ahem/third_party/MNN -B target/mnn-host -G Ninja -DCMAKE_BUILD_TYPE=Release -DMNN_BUILD_LLM=ON -DMNN_LOW_MEMORY=ON -DMNN_BUILD_CONVERTER=OFF -DMNN_BUILD_TOOLS=OFF -DMNN_BUILD_QUANTOOLS=OFF -DMNN_LLM_BUILD_DEMO=OFF -DMNN_OPENCL=OFF -DMNN_KLEIDIAI=OFF -DMNN_SME2=OFF -DMNN_ARM82=ON -DLLM_SUPPORT_HTTP_RESOURCE=OFF
cmake --build target/mnn-host --target llm -j 6
