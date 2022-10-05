
```
cargo build --target i686-pc-windows-msvc
```

Need a Windows linker to link, I grabbed lld here:
https://chromium.googlesource.com/chromium/src/tools/clang/+/refs/heads/main/scripts/update.py

```
$ cat ~/.cargo/config 
[target.i686-pc-windows-msvc]
linker = "lld"
```

Need to be separate from outer workspace for `panic = "abort"` bit to apply,
which is needed to disable `eh_personality`.

extern "system"
means stdcall (instead of cdecl)

