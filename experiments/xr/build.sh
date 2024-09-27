#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# ============================
# Configuration Section
# ============================

# Path to your Android NDK installation
# Replace this with the actual path to your NDK
# ANDROID_NDK_HOME="/Users/puppy/Library/Android/sdk/ndk/26.1.10909125"

# Android API level you are targeting
ANDROID_API_LEVEL=21

# Target architecture
TARGET_ARCH=aarch64-linux-android

# Number of parallel jobs for make (adjust as needed)
MAKE_JOBS=4

# Path to the CMake toolchain file provided by the NDK
CMAKE_TOOLCHAIN_FILE="$ANDROID_NDK_ROOT/build/cmake/android.toolchain.cmake"

# Path to the NDK's compiler binaries
TOOLCHAIN_BIN="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64/bin"

# ============================
# Verify NDK Path
# ============================

if [ ! -d "$ANDROID_NDK_ROOT" ]; then
    echo "Error: ANDROID_NDK_ROOT directory does not exist."
    echo "Please set ANDROID_NDK_ROOT to the correct NDK path."
    exit 1
fi

if [ ! -f "$CMAKE_TOOLCHAIN_FILE" ]; then
    echo "Error: CMake toolchain file not found at $CMAKE_TOOLCHAIN_FILE"
    exit 1
fi

# ============================
# Export Environment Variables
# ============================

export ANDROID_NDK_ROOT
export PATH="$TOOLCHAIN_BIN:$PATH"

# Define the compiler paths based on the NDK's naming conventions
export CC="$TOOLCHAIN_BIN/${TARGET_ARCH}${ANDROID_API_LEVEL}-clang"
export CXX="$TOOLCHAIN_BIN/${TARGET_ARCH}${ANDROID_API_LEVEL}-clang++"

# Verify that the compilers exist
if [ ! -f "$CC" ]; then
    echo "Error: C compiler not found at $CC"
    exit 1
fi

if [ ! -f "$CXX" ]; then
    echo "Error: C++ compiler not found at $CXX"
    exit 1
fi

# Optional: Export additional flags if necessary
export CFLAGS="-D__ANDROID__ -ffunction-sections -fdata-sections -fPIC"
export CXXFLAGS="-D__ANDROID__ -ffunction-sections -fdata-sections -fPIC"
export LDFLAGS="-fuse-ld=lld"

# ============================
# Clean Previous Builds
# ============================

echo "Cleaning previous build artifacts..."
cargo clean

# ============================
# Build the Project
# ============================

echo "Starting build for target $TARGET_ARCH..."

cargo build \
    --target "$TARGET_ARCH" \
    --package xr \
    --release \
    --verbose \
    --features "" \
    --no-default-features \
    --target-dir ./target

echo "Build completed successfully."
