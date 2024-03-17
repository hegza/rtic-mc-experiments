#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[rtic::app(
    device=rp2040_hal::pac,
    peripherals = false,

)]
pub mod my_app {

    use cortex_m::asm;
    use defmt::*;
    use defmt_rtt as _;
    use panic_probe as _;

    use rp2040_hal::fugit::MicrosDurationU32;
    use rp2040_hal::gpio::bank0::Gpio25;
    use rp2040_hal::gpio::{FunctionSio, Pin, PullDown, SioOutput};
    use rp2040_hal::timer::{Alarm, Alarm0};
    // Ensure we halt the program on panic (if we don't mention this crate it won't
    // be linked)

    use embedded_hal::digital::v2::OutputPin;
    // use panic_halt as _;
    use rp2040_hal::pac::{self};

    type LedOutPin = Pin<Gpio25, FunctionSio<SioOutput>, PullDown>;
    static DELAY: u32 = 100;

    #[shared]
    struct SharedResources {
        alarm: Alarm0,
        led: LedOutPin,
    }

    #[init]
    fn system_init() -> SharedResources {
        let mut device = pac::Peripherals::take().unwrap();

        // Initialization of the system clock.
        let mut watchdog = rp2040_hal::watchdog::Watchdog::new(device.WATCHDOG);

        // Configure the clocks - The default is to generate a 125 MHz system clock
        let clocks = rp2040_hal::clocks::init_clocks_and_plls(
            // External high-speed crystal on the Raspberry Pi Pico board is 12 MHz
            12_000_000u32,
            device.XOSC,
            device.CLOCKS,
            device.PLL_SYS,
            device.PLL_USB,
            &mut device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        // The single-cycle I/O block controls our GPIO pins
        let sio = rp2040_hal::Sio::new(device.SIO);

        // Set the pins to their default state
        let pins = rp2040_hal::gpio::Pins::new(
            device.IO_BANK0,
            device.PADS_BANK0,
            sio.gpio_bank0,
            &mut device.RESETS,
        );

        // Configure GPIO25 as an output
        let led_pin = pins.gpio25.into_push_pull_output();
        // Configure Timer
        let mut timer = rp2040_hal::Timer::new(device.TIMER, &mut device.RESETS, &clocks);
        let mut alarm0 = timer.alarm_0().unwrap();
        alarm0.schedule(MicrosDurationU32::millis(DELAY)).unwrap();
        alarm0.enable_interrupt();

        // This will be autogenerated later in pre-init code, but for now still need to manually unmask
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
            pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_1);
            pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_2);
        }
        SharedResources {
            alarm: alarm0,
            led: led_pin,
        }
    }

    #[task(binds = TIMER_IRQ_0 , priority = 1, shared = [alarm, led])]
    struct MyTask {
        /* local resources */
        is_high: bool,
    }
    impl RticTask for MyTask {
        fn init() -> Self {
            Self { is_high: false }
        }

        fn exec(&mut self) {
            // self.shared().led.;
            self.shared().led.lock(|led_pin| {
                if self.is_high {
                    let _ = led_pin.set_low();
                    self.is_high = false;
                } else {
                    let _ = led_pin.set_high();
                    self.is_high = true;
                }
            });

            // let _a = MyTask2::spawn(1);

            self.shared().alarm.lock(|alarm0| {
                let _ = alarm0.schedule(MicrosDurationU32::millis(DELAY));
                alarm0.clear_interrupt();
            });
        }
    }

    // #[task(priority = 2, shared = [led])]
    // struct MyTask2;
    // impl RticSwTask for MyTask2 {
    //     type SpawnInput = u8;
    //     fn init() -> Self {
    //         Self
    //     }
    //
    //     fn exec(&mut self, _input: u8) {
    //         self.shared().led.lock(|_led| {
    //
    //         })
    //     }
    // }
    //
    //
    // #[task(priority = 2, shared = [led])]
    // struct MyTask7;
    // impl RticSwTask for MyTask7 {
    //     type SpawnInput = u8;
    //     fn init() -> Self {
    //         Self
    //     }
    // 
    //     fn exec(&mut self, _input: u8) {}
    // }

    #[task(binds = TIMER_IRQ_2 , priority = 3, shared = [alarm])]
    struct MyTask3;
    impl RticTask for MyTask3 {
        fn init() -> Self {
            Self
        }

        fn exec(&mut self) {}
    }

    #[idle]
    struct MyIdleTask {
        /* local resources */
        count: u32,
    }
    impl RticIdleTask for MyIdleTask {
        fn init() -> Self {
            Self { count: 0 }
        }

        fn exec(&mut self) -> ! {
            loop {
                self.count += 1;
                info!("looping in idle... {}", self.count);
                asm::delay(120000000);
            }
        }
    }
}
