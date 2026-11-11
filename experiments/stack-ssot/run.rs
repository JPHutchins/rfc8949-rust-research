#[path = "ssot.rs"] mod ssot;
use ssot::*;
fn main() {
    println!("Decoder sizes (Leaf, Top, Envelope<Top>, u32): {:?} bytes; Frame = {} bytes", SIZES, core::mem::size_of::<Frame>());
    let mut d = Decoder::<Envelope<Top>> { stack: core::mem::MaybeUninit::uninit() };
    println!("frames view len for Envelope<Top>: {}", frames(&mut d).len());
}
