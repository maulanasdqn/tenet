# rkm-sec (native device-guard library)

A native `.so` device-guard modeled on the pattern Shopee's Android app uses (`libshpsec`-style):
the security-critical logic lives in a compiled native library, not in Kotlin/Java where it is
trivial to decompile. It is a starting point for RKM Market's mobile hardening, and the fixture
`tenet-mobile` reverse-engineers.

## What it does

- **JNI bridge** — exports `Java_co_id_rkm_security_DeviceGuard_{deviceToken,signRequest,integrityCheck,pinnedCert}`.
- **Device fingerprint** (`rkm_device_fingerprint`) and **request signing** (`rkm_sign_request`,
  keyed by a secret embedded in native code).
- **Tamper hardening** — root detection (`magisk`, `su`), emulator detection (`goldfish`, `ranchu`),
  and a pinned certificate hash.
- The signing key and report endpoint live inside the binary, not in readable Kotlin.

## Build (Android arm64-v8a ELF)

Compiled inside a Debian container so it works from any host:

```bash
docker run --rm -v "$PWD":/w -w /w debian:bookworm-slim bash -c "
  apt-get update -qq && apt-get install -y -qq gcc-aarch64-linux-gnu
  aarch64-linux-gnu-gcc -shared -fPIC -O2 -Wl,-soname,librkmsec.so -o librkmsec.so rkm_sec.c"
```

In a real app you would build all ABIs (`arm64-v8a`, `armeabi-v7a`, `x86_64`) with the Android NDK
and let Gradle place them under `src/main/jniLibs/`. Refresh the tenet-mobile fixtures by copying the
`.so` (and a zip of `lib/arm64-v8a/librkmsec.so` as `rkm-market.apk`) into
`tenet-mobile/tests/fixtures/`.

## Using it from Kotlin (Android)

```kotlin
package co.id.rkm.security

object DeviceGuard {
    init { System.loadLibrary("rkmsec") }
    external fun deviceToken(seed: ByteArray, len: Int): Long
    external fun signRequest(body: ByteArray, len: Int): Long
    external fun integrityCheck(): Int
    external fun pinnedCert(): String
}
// send signRequest(body) as an X-RKM-Sign header; the server re-computes and compares.
```

## Using it from Swift (iOS)

iOS does not use `.so`/JNI. The same C compiles into a static library or an XCFramework and is called
through a bridging header:

```swift
// rkm_sec.h exposed to Swift via the bridging header
let token = rkm_device_fingerprint(seed, Int32(seed.count))
let sig = rkm_sign_request(body, Int32(body.count))
```

Build the same `rkm_sec.c` with the iOS SDK (`xcrun --sdk iphoneos clang -arch arm64 -c`) and link the
`.a` into the app, or wrap it as an XCFramework.

## The honest limit

Native code only *hides* the algorithm and raises the cost of reversing it — `tenet-mobile` still
recovers the JNI surface, the embedded strings, and the behaviour. As the Shopee investigation showed,
the real lock is **server-side**: the app signs each request with the native key, and the server
re-computes the signature and scores the device. The native lib buys obfuscation; the server-side
verification is what actually holds.
