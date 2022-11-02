#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

fn main() {
    let _parse: u32 = strp::parse!("{}");

    let _scan: (u32, u32) = strp::scan!("{}, {}");
}

#[panic_handler]
#[no_mangle]
unsafe fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    todo!()
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: layout {layout:?}");
}

struct Allocator {}

unsafe impl core::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        todo!()
    }
}

#[global_allocator]
static TEST: Allocator = Allocator {};
