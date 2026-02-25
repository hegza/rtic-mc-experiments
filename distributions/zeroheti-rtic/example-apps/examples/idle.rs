#![no_std]
#![no_main]

use bsp::rt as _;

#[rtic::app(device = bsp)]
mod app {
    use core::arch::asm;

    use bsp::{
        CPU_FREQ_HZ, apb_uart::ApbUart, mmap::apb_timer::TIMER0_ADDR, sprintln, timer_group::Timer,
    };
    use fugit::ExtU32;

    #[shared]
    struct Shared {
        dummy: bool,
    }

    #[init]
    fn init() -> Shared {
        Shared { dummy: true }
    }

    #[idle]
    struct Idle {}
    impl RticIdleTask for Idle {
        fn init() -> Self {
            Self {}
        }
        fn exec(&mut self) -> ! {
            loop {
                sprintln!("enter sleep");
                unsafe { asm!("wfi") };
            }
        }
    }

    #[task(binds = Timer0Cmp, priority=1, shared=[dummy])]
    struct SomeTask {}

    impl RticTask for SomeTask {
        fn init() -> Self {
            let _uart = ApbUart::init(CPU_FREQ_HZ, 115_200);
            sprintln!("init");

            let mut timer = Timer::init::<TIMER0_ADDR>().into_periodic();
            timer.set_period(50.micros());
            timer.start();

            Self {}
        }

        fn exec(&mut self) {
            sprintln!("A");
            sprintln!("1");
            sprintln!("B");
        }
    }
}
