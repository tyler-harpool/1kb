#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Gate function: returns 1 to signal WASM loaded successfully.
/// Used by JS to conditionally render the page content.
#[unsafe(no_mangle)]
pub extern "C" fn o() -> i32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_returns_one() {
        assert_eq!(o(), 1);
    }
}
