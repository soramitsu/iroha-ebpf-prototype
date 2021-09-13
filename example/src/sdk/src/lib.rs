#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use core::alloc::GlobalAlloc;

extern crate alloc;

struct Alloc;

#[global_allocator]
static ALLOC: Alloc = Alloc;

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {}
}

// Need to provide a tiny `panic` implementation for `#![no_std]`.
// This translates into an `unreachable` instruction that will
// raise a `trap` the WebAssembly execution if we panic at runtime.
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

mod syscalls {
    extern "C" {
        pub fn sol_log_(account: *const u8, account_len: usize, amount: u64);
        pub fn sol_log_64_(account: *const u8, account_len: usize, amount: u64);
        pub fn sol_log_compute_units_(account: *const u8, account_len: usize, amount: *mut u64);
    }
}

#[inline(always)]
pub fn mint(account: &str, amount: u64) {
    unsafe { syscalls::sol_log_(account.as_ptr(), account.len(), amount) }
}

#[inline(always)]
pub fn burn(account: &str, amount: u64) {
    unsafe { syscalls::sol_log_64_(account.as_ptr(), account.len(), amount) }
}

#[inline(always)]
pub fn balance(account: &str) -> u64 {
    let mut x = 0;
    {
        let amount = &mut x as *mut u64;
        unsafe { syscalls::sol_log_compute_units_(account.as_ptr(), account.len(), amount) };
    }
    x
}

#[macro_export]
macro_rules! entrypoint {
    ( $ent:ident ) => {
        #[no_mangle]
        pub extern "C" fn entrypoint(account_name: *const u8) {
            unsafe {
                let len = account_name as *const [u8; 4];
                let len: [u8; 4] = core::ptr::read(len);
                let len = u32::from_le_bytes(len);
                let name = std::slice::from_raw_parts(account_name.offset(4), len as usize);
                let name = core::str::from_utf8_unchecked(name);
                $ent(name)
            }
        }
    };
}
