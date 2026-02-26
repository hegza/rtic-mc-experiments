#![no_std]
#![no_main]

use bsp::rt as _;

#[rtic::app(device=bsp, dispatchers = [Interrupt2, Interrupt3])]
mod app {
    use bsp::{
        apb_uart::ApbUart, mmap::apb_timer::TIMER0_ADDR, sprintln, timer_group::Timer, CPU_FREQ_HZ,
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

    #[task(binds = Timer0Cmp, priority=1, shared=[dummy])]
    struct SomeTask {}

    impl RticTask for SomeTask {
        fn init() -> Self {
            let _uart = ApbUart::init(CPU_FREQ_HZ, 115_200);
            sprintln!("init");

            let mut timer = Timer::init::<TIMER0_ADDR>().into_periodic();
            timer.set_period(10.micros());
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
