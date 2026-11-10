#![no_std]
#![no_main]

use core::hint::black_box;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut buf = [0u8; 128];
    let mut scratch = [0u8; 96];
    let written = probe::encode_chunk(&black_box(probe::sample()), &mut buf).unwrap_or(0);
    black_box(probe::decode_chunk(black_box(&buf[..written]), &mut scratch).is_ok());
    loop {
        core::hint::spin_loop()
    }
}
