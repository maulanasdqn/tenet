#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

const HEAP_SIZE: usize = 1 << 16;

struct Bump {
    offset: UnsafeCell<usize>,
    heap: UnsafeCell<[u8; HEAP_SIZE]>,
}

unsafe impl Sync for Bump {}

unsafe impl GlobalAlloc for Bump {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let offset = &mut *self.offset.get();
        let start = (*offset + align - 1) & !(align - 1);
        let end = start + layout.size();
        if end > HEAP_SIZE {
            return core::ptr::null_mut();
        }
        *offset = end;
        (self.heap.get() as *mut u8).add(start)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: Bump = Bump {
    offset: UnsafeCell::new(0),
    heap: UnsafeCell::new([0; HEAP_SIZE]),
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const MARKERS: [&str; 6] = [
    "rkm-sec/v1",
    "x-rkm-device",
    "RKM_DEVICE_TOKEN",
    "canvas.hash",
    "webgl.renderer",
    "navigator.userAgent",
];

const TOKEN_KEY: u64 = 0x9e37_79b9_7f4a_7c15;

#[link(wasm_import_module = "rkm_host")]
extern "C" {
    fn host_now_ms() -> u64;
    fn host_entropy() -> u64;
}

#[no_mangle]
pub extern "C" fn rkm_alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::with_capacity(len);
    let ptr = buffer.as_mut_ptr();
    core::mem::forget(buffer);
    ptr
}

#[no_mangle]
pub extern "C" fn rkm_version() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn rkm_fingerprint(ptr: *const u8, len: usize) -> u64 {
    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
    let mut state: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x0000_0100_0000_01b3);
        state = state.rotate_left(13);
    }
    for marker in MARKERS {
        for byte in marker.as_bytes() {
            state = state.wrapping_add(u64::from(*byte)).rotate_left(7);
        }
    }
    state
}

#[no_mangle]
pub extern "C" fn rkm_device_token(fingerprint: u64) -> u64 {
    let now = unsafe { host_now_ms() };
    let salt = unsafe { host_entropy() };
    let mut token = fingerprint ^ TOKEN_KEY;
    token = token.wrapping_mul(0xff51_afd7_ed55_8ccd) ^ now.rotate_left(17);
    token = token.wrapping_add(salt).rotate_left(29);
    token ^= token >> 33;
    token
}

#[no_mangle]
pub extern "C" fn rkm_marker(index: usize) -> *const u8 {
    MARKERS[index % MARKERS.len()].as_ptr()
}
