#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Plasma effect: computes pixel color from position and time.
/// Used to drive a canvas animation — each pixel's green channel
/// is determined by (x * y) ^ t, clamped to 0-255.
#[unsafe(no_mangle)]
pub extern "C" fn f(x: i32, y: i32, t: i32) -> i32 {
    (x.wrapping_mul(y) ^ t) & 0xFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_at_zero_time_is_zero() {
        assert_eq!(f(0, 0, 0), 0);
    }

    #[test]
    fn xor_with_time_shifts_pattern() {
        let base = f(5, 7, 0);
        let shifted = f(5, 7, 1);
        assert_ne!(base, shifted);
    }

    #[test]
    fn output_clamped_to_byte() {
        for x in [0, 40, 79] {
            for y in [0, 20, 39] {
                for t in [0, 100, 255, 1000] {
                    let v = f(x, y, t);
                    assert!(v >= 0 && v <= 255, "f({x},{y},{t}) = {v} out of range");
                }
            }
        }
    }

    #[test]
    fn zero_row_or_col_depends_only_on_time() {
        // x=0 or y=0 means x*y=0, so result is just t & 0xFF
        assert_eq!(f(0, 15, 42), 42);
        assert_eq!(f(30, 0, 99), 99);
    }
}
