#!/bin/bash
# Baut das APK ohne Gradle: cargo (arm64) → javac → d8 → aapt2 → zipalign → apksigner.
# Aufruf: android-port/scripts/build.sh [release|debug] [install]
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh

PROFILE="${1:-debug}"
BT="$ANDROID_HOME/build-tools/36.0.0"
JAR="$ANDROID_HOME/platforms/android-34/android.jar"
OUT=build
APK="$OUT/happy-kitten.apk"

if [ "$PROFILE" = "release" ]; then
  cargo build --release --target aarch64-linux-android -p happy-kitten-android
  SO="target/aarch64-linux-android/release/libhappy_kitten.so"
else
  cargo build --target aarch64-linux-android -p happy-kitten-android
  SO="target/aarch64-linux-android/debug/libhappy_kitten.so"
fi

rm -rf "$OUT/classes" "$OUT/dex" "$OUT/res" "$OUT/stage"
mkdir -p "$OUT/classes" "$OUT/dex" "$OUT/res" "$OUT/stage/lib/arm64-v8a"

javac --release 11 -Xlint:-options -cp "$JAR" -d "$OUT/classes" java/de/gottpot/happykitten/*.java
"$BT/d8" --release --min-api 29 --lib "$JAR" --output "$OUT/dex" $(find "$OUT/classes" -name '*.class')

"$BT/aapt2" compile --dir res -o "$OUT/res/res.zip"
"$BT/aapt2" link -I "$JAR" --manifest AndroidManifest.xml --min-sdk-version 29 --target-sdk-version 34 \
  --version-code 1 --version-name "$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)" \
  -o "$OUT/unsigned.apk" "$OUT/res/res.zip"

cp "$SO" "$OUT/stage/lib/arm64-v8a/libhappy_kitten.so"
"$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip" --strip-debug "$OUT/stage/lib/arm64-v8a/libhappy_kitten.so"
cp "$OUT/dex/classes.dex" "$OUT/stage/classes.dex"
(cd "$OUT/stage" && zip -q -0 -r ../unsigned.apk lib classes.dex)

"$BT/zipalign" -f -p 4 "$OUT/unsigned.apk" "$OUT/aligned.apk"
KS="$OUT/debug.keystore"
[ -f "$KS" ] || keytool -genkeypair -keystore "$KS" -storepass android -keypass android \
  -alias hk -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Happy-Kitten Debug" >/dev/null 2>&1
"$BT/apksigner" sign --ks "$KS" --ks-pass pass:android --key-pass pass:android --out "$APK" "$OUT/aligned.apk"
echo "APK: $APK ($(du -h "$APK" | cut -f1))"

if [ "${2:-}" = "install" ]; then
  adb install -r "$APK"
  adb shell am start -n de.gottpot.happykitten/.MainActivity
fi
