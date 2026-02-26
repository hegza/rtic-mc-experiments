#![allow(clippy::inline_always)]

pub use bsp::riscv_pac::InterruptNumber;
pub use bsp::Peripherals;

/// Distribution crate must re-export the `export` module from all the used compilation passes
pub use rtic_sw_pass::export::*;

/// Exports required by core-pass
#[cfg(feature = "intc-hetic")]
pub use bsp::hetic::InterruptNumber; // a trait that abstracts an interrupt type

/// Re-exports needed from the code generation in internal rtic-macro crate
pub use bsp::register::mintthresh;

pub mod interrupts {
    pub use bsp::interrupt::CoreInterrupt::*;
    pub use bsp::interrupt::ExternalInterrupt::*;
}
pub use bsp::interrupt::{CoreInterrupt, ExternalInterrupt};
pub use bsp::nested_interrupt;
pub use bsp::riscv::interrupt::machine::{
    disable as interrupt_disable, enable as interrupt_enable,
};

/// Lock implementation using threshold and global Critical Section (CS)
///
/// # Safety
///
/// The system ceiling is raised from current to ceiling
/// by either
/// - raising the threshold to the ceiling value, or
/// - disable all interrupts in case we want to
///   mask interrupts with maximum priority
///
/// Dereferencing a raw pointer inside CS
///
/// The priority.set/priority.get can safely be outside the CS
/// as being a context local cell (not affected by preemptions).
/// It is merely used in order to omit masking in case current
/// priority is current priority >= ceiling.
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, priority: u8, ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    // Save mintthresh
    let current = mintthresh::write((ceiling as usize).into());

    let r = f(unsafe { &mut *ptr });

    // Restore mintthresh
    mintthresh::write((current as usize).into());

    r
}

/// Sets the given software interrupt as pending
pub fn pend<T: InterruptNumber>(irq: T) {
    #[cfg(feature = "intc-clic")]
    unsafe {
        bsp::clic::CLIC::ip(irq).pend()
    };
    #[cfg(feature = "intc-hetic")]
    bsp::hetic::Hetic::line(irq.number()).pend();
    #[cfg(feature = "intc-edfic")]
    bsp::edfic::Edfic::line(irq.number()).pend();
}

/// Sets the given software interrupt as not pending
pub fn unpend<T: InterruptNumber>(irq: T) {
    #[cfg(feature = "intc-clic")]
    unsafe {
        bsp::clic::CLIC::ip(irq).unpend()
    };
    #[cfg(feature = "intc-hetic")]
    bsp::hetic::Hetic::line(irq.number()).unpend();
    #[cfg(feature = "intc-edfic")]
    bsp::edfic::Edfic::line(irq.number()).unpend();
}

pub fn enable<T: InterruptNumber>(
    irq: T,
    // HACK: enable accepts 32-bit wide parameter to support EDFIC, that will be
    // silenty truncated for CLIC and HETIC
    level: u32,
) {
    #[cfg(feature = "intc-clic")]
    {
        use bsp::clic::{Clic, Polarity, Trig};

        Clic::attr(irq).set_trig(Trig::Edge);
        Clic::attr(irq).set_polarity(Polarity::Pos);
        Clic::attr(irq).set_shv(true);
        Clic::ctl(irq).set_level(unsafe { level.try_into().unwrap_unchecked() });
        unsafe { Clic::ie(irq).enable() };
    }
    #[cfg(feature = "intc-hetic")]
    {
        use bsp::hetic::Hetic;

        Hetic::line(irq.number()).set_level_prio(unsafe { level.try_into().unwrap_unchecked() });
        Hetic::line(irq.number()).enable();
    }
    #[cfg(feature = "intc-edfic")]
    {
        use bsp::edfic::{Edfic, Pol, Trig};

        Edfic::line(irq.number()).set_pol(Pol::Pos);
        Edfic::line(irq.number()).set_trig(Trig::Edge);
        Edfic::line(irq.number()).enable();
        // HACK: level parameter sets deadline. ??? :D maybe fix using a
        // compiler pass for crazy hardware or something.
        // Convert to deadline
        let dl = (255 - level) << 8;
        Edfic::line(irq.number()).set_dl(dl);
    }
}
