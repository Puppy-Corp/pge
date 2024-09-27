#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

cargo build --target aarch64-linux-android --package xr
echo "Copying the shared library to the Android project"
cp ./target/aarch64-linux-android/debug/libxr.so ./experiments/xr/android/app/src/main/jniLibs/arm64-v8a