#!/bin/bash
# Baut das APK ohne Gradle: cargo (arm64) → javac → d8 → aapt2 → zipalign → apksigner.
# Aufruf: android-port/scripts/build.sh [release|debug] [install]
#
# debug   → build/happy-kitten.apk, signiert mit einem Debug-Schlüssel (nur zum Ausprobieren)
# release → build/Happy-Kitten-<Version>-Android-arm64.apk, signiert mit dem Release-Schlüssel
#           aus ~/.happy-kitten-signing/ (release.keystore + keystore.pass, nie im Repo).
#           Anderer Ort: HK_KEYSTORE, HK_KEYSTORE_PASS_FILE, HK_KEY_ALIAS.
# Der Versionscode steigt mit der Version (1.2.3 → 10203). Android installiert ein Update nur,
# wenn sein Code größer ist als der der installierten App: bei jedem Release die Version erhöhen.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh

PROFILE="${1:-debug}"
BT="$ANDROID_HOME/build-tools/36.0.0"
JAR="$ANDROID_HOME/platforms/android-34/android.jar"
OUT=build
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
IFS=. read -r V_MAJOR V_MINOR V_PATCH <<<"$VERSION"
VERSION_CODE=$((V_MAJOR * 10000 + V_MINOR * 100 + V_PATCH))
if [ "$PROFILE" = "release" ]; then
  APK="$OUT/Happy-Kitten-$VERSION-Android-arm64.apk"
  KS="${HK_KEYSTORE:-$HOME/.happy-kitten-signing/release.keystore}"
  KS_PASS_FILE="${HK_KEYSTORE_PASS_FILE:-$HOME/.happy-kitten-signing/keystore.pass}"
  KEY_ALIAS="${HK_KEY_ALIAS:-happy-kitten}"
  if [ ! -f "$KS" ] || [ ! -f "$KS_PASS_FILE" ]; then
    echo "Release-Schlüssel fehlt ($KS)." >&2
    echo "Ohne ihn lässt sich später kein Update über die installierte Version legen." >&2
    exit 1
  fi
else
  APK="$OUT/happy-kitten.apk"
fi

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
  --version-code "$VERSION_CODE" --version-name "$VERSION" \
  -o "$OUT/unsigned.apk" "$OUT/res/res.zip"

cp "$SO" "$OUT/stage/lib/arm64-v8a/libhappy_kitten.so"
STRIP="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip"
if [ "$PROFILE" = "release" ]; then
  "$STRIP" --strip-all "$OUT/stage/lib/arm64-v8a/libhappy_kitten.so"
else
  "$STRIP" --strip-debug "$OUT/stage/lib/arm64-v8a/libhappy_kitten.so"
fi
cp "$OUT/dex/classes.dex" "$OUT/stage/classes.dex"
(cd "$OUT/stage" && zip -q -0 -r ../unsigned.apk lib classes.dex)

"$BT/zipalign" -f -p 4 "$OUT/unsigned.apk" "$OUT/aligned.apk"
if [ "$PROFILE" = "release" ]; then
  "$BT/apksigner" sign --ks "$KS" --ks-key-alias "$KEY_ALIAS" \
    --ks-pass "file:$KS_PASS_FILE" \
    --out "$APK" "$OUT/aligned.apk"
else
  KS="$OUT/debug.keystore"
  [ -f "$KS" ] || keytool -genkeypair -keystore "$KS" -storepass android -keypass android \
    -alias hk -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Happy-Kitten Debug" >/dev/null 2>&1
  "$BT/apksigner" sign --ks "$KS" --ks-pass pass:android --key-pass pass:android \
    --out "$APK" "$OUT/aligned.apk"
fi
"$BT/apksigner" verify --print-certs "$APK" 2>/dev/null | grep -E "SHA-256|Signer #1 certificate DN" || true
echo "APK: $APK ($(du -h "$APK" | cut -f1))"

if [ "${2:-}" = "install" ]; then
  adb install -r "$APK" || {
    echo "Tipp: Wurde die App vorher mit einem anderen Schlüssel installiert (debug ↔ release)?" >&2
    echo "      Dann erst deinstallieren: adb uninstall de.gottpot.happykitten" >&2
    exit 1
  }
  adb shell am start -n de.gottpot.happykitten/.MainActivity
fi
