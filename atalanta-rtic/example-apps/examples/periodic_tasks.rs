#![no_main]
#![no_std]
#![allow(static_mut_refs)]
#![allow(non_snake_case)]

#[rtic::app(device = bsp)]
mod app {

    use core::arch::asm;

    use bsp::{
        clic::{Clic, Polarity, Trig},
        embedded_io::Write,
        mtimer::{self, MTimer},
        riscv, sprint, sprintln,
        tb::signal_pass,
        timer_group::{Timer0, Timer1, Timer2, Timer3},
        uart::*,
        Interrupt, CPU_FREQ,
    };
    use ufmt::derive::uDebug;

    #[shared]
    struct Shared {}

    #[cfg_attr(feature = "ufmt", derive(uDebug))]
    #[cfg_attr(not(feature = "ufmt"), derive(Debug))]
    struct TaskDef {
        // level is specified in RTIC task
        level: u8,
        period_us: u32,
        duration_us: u32,
        start_offset_us: u32,
    }

    const TEST_DURATION: mtimer::Duration = mtimer::Duration::micros(1_000);

    impl TaskDef {
        pub const fn new(
            level: u8,
            period_us: u32,
            duration_us: u32,
            start_offset_us: u32,
        ) -> Self {
            Self {
                period_us,
                duration_us,
                level,
                start_offset_us,
            }
        }
    }

    const TEST_BASE_PERIOD_US: u32 = 100;
    const TASK0: TaskDef = TaskDef::new(
        1,
        TEST_BASE_PERIOD_US / 1,
        /* 20 % */ TEST_BASE_PERIOD_US / 5,
        /* 10 % */ TEST_BASE_PERIOD_US / 10,
    );
    const TASK1: TaskDef = TaskDef::new(
        2,
        TEST_BASE_PERIOD_US / 1,
        /* 10 % */ TEST_BASE_PERIOD_US / 10,
        /* 60 % */ 3 * TEST_BASE_PERIOD_US / 5,
    );
    const TASK2: TaskDef = TaskDef::new(
        3,
        TEST_BASE_PERIOD_US / 2,
        /* 5 % */ TEST_BASE_PERIOD_US / 20,
        /* 37.5 % */ 3 * TEST_BASE_PERIOD_US / 8,
    );
    const TASK3: TaskDef = TaskDef::new(
        4,
        TEST_BASE_PERIOD_US / 4,
        /* 2.5 % */ TEST_BASE_PERIOD_US / 40,
        /* 12.5 % */ TEST_BASE_PERIOD_US / 8,
    );
    const PERIPH_CLK_DIV: u64 = 2;
    const CYCLES_PER_SEC: u64 = CPU_FREQ as u64 / PERIPH_CLK_DIV;
    const CYCLES_PER_MS: u64 = CYCLES_PER_SEC / 1_000;
    const CYCLES_PER_US: u64 = CYCLES_PER_MS / 1_000;

    static mut TASK0_COUNT: usize = 0;
    static mut TASK1_COUNT: usize = 0;
    static mut TASK2_COUNT: usize = 0;
    static mut TASK3_COUNT: usize = 0;

    const USE_PCS: bool = false;

    #[init]
    fn init() -> Shared {
        let mut serial = ApbUart::init(CPU_FREQ, 115_200);
        sprintln!("[periodic_tasks (PCS={:?})]", USE_PCS);

        sprintln!(
            "Tasks: \r\n  {:?}\r\n  {:?}\r\n  {:?}\r\n  {:?}",
            TASK0,
            TASK1,
            TASK2,
            TASK3
        );
        sprintln!("Test duration: {} us", TEST_DURATION.to_micros());

        // Pre-enable interrupts; required for behavior match
        rtic::export::enable(Interrupt::Timer0Cmp, TASK0.level);
        rtic::export::enable(Interrupt::Timer1Cmp, TASK1.level);
        rtic::export::enable(Interrupt::Timer2Cmp, TASK2.level);
        rtic::export::enable(Interrupt::Timer3Cmp, TASK3.level);
        rtic::export::enable(Interrupt::MachineTimer, u8::MAX);

        if USE_PCS {
            Clic::ie(Interrupt::Timer0Cmp).set_pcs(true);
            Clic::ie(Interrupt::Timer1Cmp).set_pcs(true);
            Clic::ie(Interrupt::Timer2Cmp).set_pcs(true);
            Clic::ie(Interrupt::Timer3Cmp).set_pcs(true);
        }

        // Make sure serial is done printing before proceeding to the test case
        unsafe { serial.flush().unwrap_unchecked() };

        // Use mtimer for timeout
        let mut mtimer = MTimer::instance().into_oneshot();

        let mut timers = (
            Timer0::init(),
            Timer1::init(),
            Timer2::init(),
            Timer3::init(),
        );

        timers.0.set_cmp(TASK0.period_us * CYCLES_PER_US as u32);
        timers
            .0
            .set_counter((TASK0.period_us - TASK0.start_offset_us) * CYCLES_PER_US as u32);
        timers.1.set_cmp(TASK1.period_us * CYCLES_PER_US as u32);
        timers
            .1
            .set_counter((TASK1.period_us - TASK1.start_offset_us) * CYCLES_PER_US as u32);
        timers.2.set_cmp(TASK2.period_us * CYCLES_PER_US as u32);
        timers
            .2
            .set_counter((TASK2.period_us - TASK2.start_offset_us) * CYCLES_PER_US as u32);
        timers.3.set_cmp(TASK3.period_us * CYCLES_PER_US as u32);
        timers
            .3
            .set_counter((TASK3.period_us - TASK3.start_offset_us) * CYCLES_PER_US as u32);

        // --- Test critical ---
        unsafe { asm!("fence") };

        // Test will end when MachineTimer fires
        mtimer.start(TEST_DURATION);
        timers.0.enable();
        timers.1.enable();
        timers.2.enable();
        timers.3.enable();

        // Pre-enable interrupts; required for behavior match
        unsafe { riscv::interrupt::enable() };

        Shared {}
    }

    #[task(binds = Timer0Cmp, priority=1)]
    struct T0 {}
    #[task(binds = Timer1Cmp, priority=2)]
    struct T1 {}
    #[task(binds = Timer2Cmp, priority=3)]
    struct T2 {}
    #[task(binds = Timer3Cmp, priority=4)]
    struct T3 {}

    impl RticTask for T0 {
        fn init() -> Self {
            Self {}
        }

        fn exec(&mut self) {
            let mtimer = MTimer::instance().into_lo();
            let sample = mtimer.counter();
            unsafe { TASK0_COUNT += 1 };
            let task_end = sample + TASK0.duration_us * CYCLES_PER_US as u32;
            while mtimer.counter() <= task_end {}
        }
    }

    impl RticTask for T1 {
        fn init() -> Self {
            Self {}
        }

        fn exec(&mut self) {
            let mtimer = MTimer::instance().into_lo();
            let sample = mtimer.counter();
            unsafe { TASK1_COUNT += 1 };
            let task_end = sample + TASK0.duration_us * CYCLES_PER_US as u32;
            while mtimer.counter() <= task_end {}
        }
    }

    impl RticTask for T2 {
        fn init() -> Self {
            Self {}
        }

        fn exec(&mut self) {
            let mtimer = MTimer::instance().into_lo();
            let sample = mtimer.counter();
            unsafe { TASK2_COUNT += 1 };
            let task_end = sample + TASK0.duration_us * CYCLES_PER_US as u32;
            while mtimer.counter() <= task_end {}
        }
    }

    impl RticTask for T3 {
        fn init() -> Self {
            Self {}
        }

        fn exec(&mut self) {
            let mtimer = MTimer::instance().into_lo();
            let sample = mtimer.counter();
            unsafe { TASK3_COUNT += 1 };
            let task_end = sample + TASK0.duration_us * CYCLES_PER_US as u32;
            while mtimer.counter() <= task_end {}
        }
    }

    #[task(binds = MachineTimer, priority=0xff)]
    struct Timeout {}

    impl RticTask for Timeout {
        fn init() -> Self {
            Self {}
        }

        fn exec(&mut self) {
            riscv::interrupt::disable();
            unsafe { asm!("fence") };
            // --- Test critical end ---

            let mut timer = MTimer::instance();
            timer.disable();

            unsafe {
                // Draw mtimer to max value to make sure all currently pending or in flight
                // TimerXCmp interrupts fall through.
                timer.set_counter(u64::MAX);

                // Disable all timers & interrupts, so no more instances will fire
                Timer0::instance().disable();
                Timer1::instance().disable();
                Timer2::instance().disable();
                Timer3::instance().disable();
                Clic::ip(Interrupt::MachineTimer).unpend();
                Clic::ip(Interrupt::Timer0Cmp).unpend();
                Clic::ip(Interrupt::Timer1Cmp).unpend();
                Clic::ip(Interrupt::Timer2Cmp).unpend();
                Clic::ip(Interrupt::Timer3Cmp).unpend();
            }
            // Clean up (RTIC won't do this for us unfortunately)
            tear_irq(Interrupt::Timer0Cmp);
            tear_irq(Interrupt::Timer1Cmp);
            tear_irq(Interrupt::Timer2Cmp);
            tear_irq(Interrupt::Timer3Cmp);
            tear_irq(Interrupt::MachineTimer);
            if USE_PCS {
                Clic::ie(Interrupt::Timer0Cmp).set_pcs(false);
                Clic::ie(Interrupt::Timer1Cmp).set_pcs(false);
                Clic::ie(Interrupt::Timer2Cmp).set_pcs(false);
                Clic::ie(Interrupt::Timer3Cmp).set_pcs(false);
            }

            let mut serial = unsafe { ApbUart::instance() };

            unsafe {
                sprintln!(
                    "Task counts:\r\n{} | {} | {} | {}",
                    TASK0_COUNT,
                    TASK1_COUNT,
                    TASK2_COUNT,
                    TASK3_COUNT
                );
                let total_in_task0 = TASK0.duration_us * TASK0_COUNT as u32;
                let total_in_task1 = TASK1.duration_us * TASK1_COUNT as u32;
                let total_in_task2 = TASK2.duration_us * TASK2_COUNT as u32;
                let total_in_task3 = TASK3.duration_us * TASK3_COUNT as u32;
                sprintln!(
                "Theoretical total duration spent in task workload (us):\r\n{} | {} | {} | {} = {}",
                total_in_task0,
                total_in_task1,
                total_in_task2,
                total_in_task3,
                total_in_task0 + total_in_task1 + total_in_task2 + total_in_task3,
            );

                // Assert that each task runs the expected number of times
                for (count, task) in &[
                    (TASK0_COUNT, TASK0),
                    (TASK1_COUNT, TASK1),
                    (TASK2_COUNT, TASK2),
                    (TASK3_COUNT, TASK3),
                ] {
                    assert_eq!(
                        *count,
                        (TEST_DURATION.to_micros() as usize + task.start_offset_us as usize)
                            / task.period_us as usize
                    )
                }
                // Make sure serial is done printing before proceeding to the next iteration
                serial.flush().unwrap_unchecked();
            }

            signal_pass(Some(&mut serial));
        }
    }

    /// Tear down the IRQ configuration to avoid side-effects for further testing
    pub fn tear_irq(irq: Interrupt) {
        Clic::ie(irq).disable();
        Clic::ctl(irq).set_level(0x0);
        Clic::attr(irq).set_shv(false);
        Clic::attr(irq).set_trig(Trig::Level);
        Clic::attr(irq).set_polarity(Polarity::Pos);
    }
}
