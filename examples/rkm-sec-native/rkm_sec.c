#include <stdint.h>
#include <string.h>

__attribute__((visibility("default"))) const char RKM_SDK_TAG[] = "rkm-sec-native/1.0";
__attribute__((visibility("default"))) const char RKM_REPORT_URL[] =
    "https://api.market.rajawalikaryamulya.co.id/v1/sec/report";
static const char RKM_HMAC_KEY[] = "RKM_NATIVE_SIGNING_KEY_v1_change_me";
static const char ROOT_MARKERS[] = "/system/bin/su:/system/xbin/su:/sbin/su:magisk:supersu";
static const char EMU_MARKERS[] = "goldfish:ranchu:vbox86:generic_x86:sdk_gphone";
static const char PIN_SHA256[] = "sha256/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

static uint64_t fnv1a(const uint8_t *p, int n) {
    uint64_t h = 0xcbf29ce484222325ULL;
    for (int i = 0; i < n; i++) { h ^= p[i]; h *= 0x100000001b3ULL; }
    return h;
}

uint64_t rkm_device_fingerprint(const uint8_t *buf, int len) {
    uint64_t h = fnv1a(buf, len);
    return h ^ fnv1a((const uint8_t *)RKM_SDK_TAG, (int)strlen(RKM_SDK_TAG));
}

uint64_t rkm_sign_request(const uint8_t *body, int len) {
    uint64_t k = fnv1a((const uint8_t *)RKM_HMAC_KEY, (int)strlen(RKM_HMAC_KEY));
    return (fnv1a(body, len) ^ k) * 0xff51afd7ed558ccdULL;
}

int rkm_is_rooted(const char *path) { return strstr(ROOT_MARKERS, path) != 0; }
int rkm_is_emulator(const char *prop) { return strstr(EMU_MARKERS, prop) != 0; }

uint64_t Java_co_id_rkm_security_DeviceGuard_deviceToken(void *e, void *t, const uint8_t *b, int n) {
    return rkm_device_fingerprint(b, n);
}
uint64_t Java_co_id_rkm_security_DeviceGuard_signRequest(void *e, void *t, const uint8_t *b, int n) {
    return rkm_sign_request(b, n);
}
int Java_co_id_rkm_security_DeviceGuard_integrityCheck(void *e, void *t) {
    return rkm_is_rooted("magisk") || rkm_is_emulator("goldfish");
}
const char *Java_co_id_rkm_security_DeviceGuard_pinnedCert(void *e, void *t) { return PIN_SHA256; }
