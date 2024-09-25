
```
cargo build --target aarch64-linux-android --package xr
```

## Macos

Linker for android need to be set in ~/.cargo/config.toml file. For example:
```
[target.aarch64-linux-android]
linker = "HOME/Library/Android/sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android21-clang"
```