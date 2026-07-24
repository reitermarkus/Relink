#[path = "common/mod.rs"]
mod fixture_support;

use std::fs;

use elf_loader::{
    Loader, Relocator, Result,
    image::{SyntheticModule, SyntheticSymbol},
};

const LOADER: Loader = Loader::new();
const RELOCATOR: Relocator = Relocator::new();
fn host_symbols() -> SyntheticModule {
    unsafe fn global_alloc(size: usize, align: usize) -> *mut u8 {
        unsafe {
            let layout = core::alloc::Layout::from_size_align_unchecked(size, align);
            std::alloc::alloc(layout)
        }
    }

    unsafe fn global_dealloc(ptr: *mut u8, size: usize, align: usize) {
        unsafe {
            let layout = core::alloc::Layout::from_size_align_unchecked(size, align);
            std::alloc::dealloc(ptr, layout)
        }
    }

    unsafe extern "C" fn rust_eh_personality() -> ! {
        println!("eh_personality");

        loop {}
    }

    pub fn println(s: &str) {
        println!("{s}")
    }

    SyntheticModule::new(
        "__host",
        [
            SyntheticSymbol::function("println", println as *const ()),
            // Rust
            SyntheticSymbol::function("global_alloc", global_alloc as *const ()),
            SyntheticSymbol::function("global_dealloc", global_dealloc as *const ()),
            SyntheticSymbol::function("rust_eh_personality", rust_eh_personality as *const ()),
            // libc
            SyntheticSymbol::function("memcpy", libc::memcpy as *const ()),
            SyntheticSymbol::function("memset", libc::memset as *const ()),
            SyntheticSymbol::function("bcmp", libc::memcmp as *const ()),
            SyntheticSymbol::function("sysconf", libc::sysconf as *const ()),
            SyntheticSymbol::function("__errno_location", libc::__errno_location as *const ()),
            SyntheticSymbol::function("mmap", libc::mmap as *const ()),
            SyntheticSymbol::function("munmap", libc::munmap as *const ()),
            SyntheticSymbol::function("madvise", libc::madvise as *const ()),
            SyntheticSymbol::function("mprotect", libc::mprotect as *const ()),
            SyntheticSymbol::function("memmove", libc::memmove as *const ()),
            // libunwind
            SyntheticSymbol::function("_Unwind_Resume", libunwind::_Unwind_Resume as *const ()),
            // SyntheticSymbol::function("pthread_key_create", libc::pthread_key_create as *const ()),
            // SyntheticSymbol::function("pthread_key_delete", libc::pthread_key_delete as *const ()),
            // SyntheticSymbol::function(
            //     "pthread_setspecific",
            //     libc::pthread_setspecific as *const (),
            // ),
            // SyntheticSymbol::function("close", libc::close as *const ()),
            // SyntheticSymbol::function("dl_iterate_phdr", libc::dl_iterate_phdr as *const ()),
            // SyntheticSymbol::function("syscall", libc::syscall as *const ()),
            // SyntheticSymbol::function("read", libc::read as *const ()),
            // SyntheticSymbol::function("open64", libc::open64 as *const ()),
            // SyntheticSymbol::function("lseek64", libc::lseek64 as *const ()),
            // SyntheticSymbol::function("fstat64", libc::fstat64 as *const ()),
            // SyntheticSymbol::function("stat64", libc::stat64 as *const ()),
            // SyntheticSymbol::function("mmap64", libc::mmap64 as *const ()),
            // SyntheticSymbol::function("abort", libc::abort as *const ()),
            // SyntheticSymbol::function("readlink", libc::readlink as *const ()),
            // SyntheticSymbol::function("realpath", libc::realpath as *const ()),
            // SyntheticSymbol::function("getcwd", libc::getcwd as *const ()),
            // SyntheticSymbol::function("_Unwind_GetIP", libunwind::_Unwind_GetIP as *const ()),
            // SyntheticSymbol::function(
            //     "_Unwind_GetTextRelBase",
            //     libunwind::_Unwind_GetTextRelBase as *const (),
            // ),
            // SyntheticSymbol::function(
            //     "_Unwind_GetDataRelBase",
            //     libunwind::_Unwind_GetDataRelBase as *const (),
            // ),
            // SyntheticSymbol::function(
            //     "_Unwind_Backtrace",
            //     libunwind::_Unwind_Backtrace as *const (),
            // ),
            // SyntheticSymbol::function("getenv", libc::getenv as *const ()),
            // SyntheticSymbol::function("strlen", libc::strlen as *const ()),
            // SyntheticSymbol::function("malloc", libc::malloc as *const ()),
            // SyntheticSymbol::function("free", libc::free as *const ()),
            // SyntheticSymbol::function("realloc", libc::realloc as *const ()),
            // SyntheticSymbol::function("write", libc::write as *const ()),
            // SyntheticSymbol::function("writev", libc::writev as *const ()),
            // SyntheticSymbol::function("read", libc::read as *const ()),
            // SyntheticSymbol::function("__xpg_strerror_r", libc::strerror_r as *const ()),
            // SyntheticSymbol::function(
            //     "_Unwind_GetLanguageSpecificData",
            //     libunwind::_Unwind_GetLanguageSpecificData as *const (),
            // ),
            // SyntheticSymbol::function(
            //     "_Unwind_GetIPInfo",
            //     libunwind::_Unwind_GetIPInfo as *const (),
            // ),
            // SyntheticSymbol::function(
            //     "_Unwind_GetRegionStart",
            //     libunwind::_Unwind_GetRegionStart as *const (),
            // ),
            // SyntheticSymbol::function("_Unwind_SetIP", libunwind::_Unwind_SetIP as *const ()),
            // SyntheticSymbol::function("_Unwind_SetGR", libunwind::_Unwind_SetGR as *const ()),
            // SyntheticSymbol::function("getenv", libc::getenv as *const ()),
        ],
    )
}

fn main() -> Result<()> {
    unsafe { std::env::set_var("RUST_LOG", "trace") };
    env_logger::init();

    let fixtures = fixture_support::ensure_all();
    let host = host_symbols();
    let libnested = RELOCATOR
        .run(
            LOADER
                .with_default_tls_resolver()
                .load_dylib(fixtures.libnested_str())?,
        )
        .scope([host])
        .relocate()?;
    let f = unsafe { libnested.get::<fn(&str, &[u8]) -> i32>("nested").unwrap() };

    let liba_str = fixtures.liba_str();
    let liba_content = fs::read(liba_str).unwrap();
    let result = f(liba_str, &liba_content);
    assert!(result == 1);
    Ok(())
}
