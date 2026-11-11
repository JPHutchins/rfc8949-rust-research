trait Depth { const MAX_DEPTH: usize; }
struct Frame(#[allow(dead_code)] u32);
struct Shallow;
impl Depth for Shallow { const MAX_DEPTH: usize = 3; }
fn decode<T: Depth, const N: usize>(stack: &mut [Frame; N]) -> usize {
    const { assert!(N >= T::MAX_DEPTH, "stack too small for T") };
    stack.len()
}
fn main() { let mut s = [Frame(0), Frame(0)]; println!("{}", decode::<Shallow, 2>(&mut s)); }
