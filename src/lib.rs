#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Square function: returns x * x.
/// Used as page gate — nonzero input = truthy result = page renders.
#[unsafe(no_mangle)]
pub extern "C" fn f(x: i32) -> i32 {
    x * x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_of_13() {
        assert_eq!(f(13), 169);
    }

    #[test]
    fn square_of_zero_is_falsy() {
        assert_eq!(f(0), 0);
    }

    #[test]
    fn square_of_negative() {
        assert_eq!(f(-5), 25);
    }
}
