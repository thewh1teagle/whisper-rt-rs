# whisper-rt-rs

Experimental, but works on Windows 11.
Install wget from winget.

```console
wget.exe https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
wget.exe https://github.com/CNugteren/CLBlast/releases/download/1.6.3/CLBlast-1.6.3-windows-x64.7z
"C:\Program Files\7-Zip\7z" x CLBlast-1.6.3-windows-x64.7z
move CLBlast-1.6.3-windows-x64 clblast
del CLBlast-1.6.3-windows-x64.7z
$env:CLBlast_DIR = "$pwd\clblast\lib\cmake\CLBlast"
$env:CMAKE_BUILD_TYPE = "RelWithDebInfo"
cp C:\vcpkg\packages\opencl_x64-windows\bin\OpenCL.dll target\debug\
cp .\clblast\bin\clblast.dll .\target\debug\
cargo run
```

With `whisper-tiny` it works in realtime on cheap hardware (amd ryzen 5 4500u). the VAD is pretty good.