#!/bin/sh
# Ensure an Android emulator is running before proceeding.

ANDROID_SDK="$HOME/Library/Android/sdk"
EMULATOR="$ANDROID_SDK/emulator/emulator"

if adb get-state >/dev/null 2>&1; then
  echo "Emulator/device already connected."
  exit 0
fi

AVD=$("$EMULATOR" -list-avds 2>/dev/null | head -1)
if [ -z "$AVD" ]; then
  echo "Error: no Android AVDs found. Create one in Android Studio first." >&2
  exit 1
fi

echo "Starting emulator: $AVD"
"$EMULATOR" -avd "$AVD" >/dev/null 2>&1 &

echo "Waiting for emulator to boot..."
adb wait-for-device
adb shell 'while [ "$(getprop sys.boot_completed)" != "1" ]; do sleep 1; done'
echo "Emulator ready."
