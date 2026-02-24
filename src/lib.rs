#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// sin(x) for x in [-PI, PI]. Horner form, 4-term Taylor.
#[unsafe(no_mangle)]
pub extern "C" fn s(x: f32) -> f32 {
    let x2 = x * x;
    x * (1.0 - x2 * (1.0 / 6.0 - x2 * (1.0 / 120.0 - x2 / 5040.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_zero() {
        assert!(s(0.0).abs() < 0.001);
    }

    #[test]
    fn sin_half_pi() {
        assert!((s(1.5707963) - 1.0).abs() < 0.01);
    }

    #[test]
    fn sin_negative_half_pi() {
        assert!((s(-1.5707963) + 1.0).abs() < 0.01);
    }

    #[test]
    fn sin_pi() {
        assert!(s(3.1415926).abs() < 0.1);
    }
}
