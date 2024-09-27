# Example xr project

Android project files are in the android directory. And it can be built with Android Studio.

## Building

```
cargo build --target aarch64-linux-android --package xr
```
Then need to copy the library to the android project.
```
cp ./target/aarch64-linux-android/debug/libxr.so ./experiments/xr/android/app/src/main/jniLibs/arm64-v8a
```

### Macos

Linker for android need to be set in ~/.cargo/config.toml file. For example:
```
[target.aarch64-linux-android]
linker = "HOME/Library/Android/sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android21-clang"
```