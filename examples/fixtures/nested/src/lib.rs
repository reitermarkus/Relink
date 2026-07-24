#![no_std]

extern crate alloc;

use alloc::format;

use core::panic::PanicInfo;

use elf_loader::{Loader, Relocator};

const LOADER: Loader = Loader::new();
const RELOCATOR: Relocator = Relocator::new();

struct ProxyAlloc;

unsafe extern "C" {
    fn global_alloc(size: usize, align: usize) -> *mut u8;
    fn global_dealloc(ptr: *mut u8, size: usize, align: usize);
}

unsafe impl core::alloc::GlobalAlloc for ProxyAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { global_alloc(layout.size(), layout.align()) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { global_dealloc(ptr, layout.size(), layout.align()) }
    }
}

#[global_allocator]
static PROXY_ALLOC: ProxyAlloc = ProxyAlloc;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe extern "Rust" {
    fn println(s: &str);
}

#[unsafe(no_mangle)]
fn nested(liba_str: &str, liba_content: &[u8]) -> i32 {
    unsafe {
        println("Loading liba...");
        println(liba_str);
    }

    let Ok(lib) = LOADER.load_dylib(liba_content) else {
        unsafe { println("load failed") };
        return -1;
    };

    unsafe {
        println("loaded");
    }

    let liba = match RELOCATOR.run(lib).relocate() {
        Ok(liba) => liba,
        Err(err) => {
            let err = format!("{}", err);
            unsafe { println(&err) };
            return -2;
        }
    };

    unsafe { println("relocated") };

    let Some(f) = (unsafe { liba.get::<fn() -> i32>("a") }) else {
        return -3;
    };

    f()
}
