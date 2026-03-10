#!/bin/sh
# Build and install the debug APK on the connected Android device/emulator.
# Detects the device ABI to build only the needed target, keeping APK size small.

set -e

ABI=$(adb shell getprop ro.product.cpu.abi)
case "$ABI" in
  arm64-v8a)    TARGET="aarch64" ; FLAVOR="arm64" ;;
  armeabi-v7a)  TARGET="armv7"   ; FLAVOR="arm"   ;;
  x86_64)       TARGET="x86_64"  ; FLAVOR="x86_64" ;;
  x86)          TARGET="i686"    ; FLAVOR="x86"    ;;
  *)            echo "Unknown ABI: $ABI" >&2; exit 1 ;;
esac

APK="src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk"
PKG="com.arktosmos.mhaoltube"

echo "Building debug APK for $ABI (target: $TARGET)..."
tauri android build --apk --debug --target "$TARGET"

echo "Installing $APK..."
if ! adb install -r "$APK"; then
  echo "Install failed, trying uninstall + install..."
  adb uninstall "$PKG" || true
  adb install "$APK"
fi

echo "Launching app..."
adb shell am start -n "$PKG/.MainActivity"
