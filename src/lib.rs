#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Population count: returns number of set bits in x.
/// Used as content gate — zero input (no hash) = 0 = page stays blank,
/// nonzero input (hash present) = positive = page renders.
#[unsafe(no_mangle)]
pub extern "C" fn f(x: i32) -> i32 {
    x.count_ones() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_length_15() {
        // "# Tyler Harpool".len() = 15, popcnt(15) = 4
        assert_eq!(f(15), 4);
    }

    #[test]
    fn empty_hash_is_falsy() {
        assert_eq!(f(0), 0);
    }

    #[test]
    fn any_nonzero_is_truthy() {
        assert!(f(1) > 0);
        assert!(f(7) > 0);
        assert!(f(100) > 0);
    }
}
