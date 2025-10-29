#![allow(clippy::inline_always)]

use bsp::hetic::{Hetic, Pol, Trig};

/// Distribution crate must re-export the `export` module from all the used compilation passes
pub use rtic_sw_pass::export::*;

/// Exports required by core-pass
pub use bsp::hetic::InterruptNumber; // a trait that abstracts an interrupt type

/// Re-exports needed from the code generation in internal rtic-macro crate
pub use bsp::register::mintthresh;

pub mod interrupts {
    pub use bsp::interrupt::CoreInterrupt::*;
    pub use bsp::interrupt::ExternalInterrupt::*;
}
pub use bsp::interrupt::{CoreInterrupt, ExternalInterrupt};
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
    if priority < ceiling {
        // Save mintthresh
        let current = mintthresh::write((ceiling as usize).into());

        let r = f(unsafe { &mut *ptr });

        // Restore mintthresh
        mintthresh::write((current as usize).into());

        r
    } else {
        f(unsafe { &mut *ptr })
    }
}

/// Sets the given software interrupt as pending
pub fn pend<T: InterruptNumber>(irq: T) {
    Hetic::line(irq.number()).pend();
}

/// Sets the given software interrupt as not pending
pub fn unpend<T: InterruptNumber>(irq: T) {
    Hetic::line(irq.number()).unpend();
}

pub fn enable<T: InterruptNumber>(irq: T, level: u8) {
    let mut line = Hetic::line(irq.number());

    line.set_trig(Trig::Edge);
    line.set_pol(Pol::Pos);
    line.set_level(level);
    line.enable();
}
