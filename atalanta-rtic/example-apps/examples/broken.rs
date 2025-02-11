#![no_std]
#![no_main]
use bsp::rt::entry;
use bsp::uart::ApbUart;
use bsp::{sprintln, CPU_FREQ};

#[entry]
fn main() -> ! {
    let _uart = ApbUart::init(CPU_FREQ, 115_200);
    sprintln!("Hi!");
    let mut q: rtic::export::Queue<u8, 2> = rtic::export::Queue::new();
    q.enqueue(8).ok();
    let c = q.dequeue().unwrap();
    sprintln!("Bye{}", c);
    loop {}
}
