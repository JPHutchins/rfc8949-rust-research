#![no_std]
#![no_main]
extern crate alloc;
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! { loop {} }
