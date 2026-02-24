#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Compute animation values for a character.
///
/// Arguments:
/// - time: current time in ms (from requestAnimationFrame)
/// - index: character index in the string
/// - effect: 0=wave, 1=glow, 2=pulse
/// - speed: animation speed multiplier (1-9)
///
/// Returns a packed u64:
/// - upper 32 bits: y_offset as f32 bits
/// - lower 32 bits: brightness (0-255) as u32
#[unsafe(no_mangle)]
pub extern "C" fn tick(time: f32, index: f32, effect: i32, speed: f32) -> u64 {
    let t = time * speed * 0.001;
    let i = index * 0.5;

    let (y, brightness) = match effect {
        // wave: characters bob up and down in a sine wave
        0 => {
            let y = sin(t + i) * 20.0;
            let b = ((sin(t * 2.0 + i) + 1.0) * 0.5 * 127.0 + 128.0) as u32;
            (y, b)
        }
        // glow: characters pulse brightness
        1 => {
            let b = ((sin(t * 3.0 + i) + 1.0) * 0.5 * 200.0 + 55.0) as u32;
            (0.0, b)
        }
        // pulse: characters scale/bounce
        _ => {
            let y = sin(t * 2.0 + i).abs() * -15.0;
            let b = ((sin(t + i * 0.3) + 1.0) * 0.5 * 127.0 + 128.0) as u32;
            (y, b)
        }
    };

    let y_bits = y.to_bits() as u64;
    let b = brightness.min(255) as u64;
    (y_bits << 32) | b
}

/// Minimal sine approximation using Taylor series.
/// Accurate enough for visual animation, tiny code size.
fn sin(x: f32) -> f32 {
    let pi = 3.14159265;
    let mut x = x % (2.0 * pi);
    if x > pi {
        x -= 2.0 * pi;
    } else if x < -pi {
        x += 2.0 * pi;
    }
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    let x9 = x7 * x2;
    x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0 + x9 / 362880.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_wave_returns_packed_value() {
        let result = tick(1000.0, 0.0, 0, 3.0);
        let y_bits = (result >> 32) as u32;
        let brightness = (result & 0xFFFFFFFF) as u32;
        let y = f32::from_bits(y_bits);
        assert!(y >= -20.0 && y <= 20.0, "y_offset out of range: {}", y);
        assert!(
            brightness >= 55 && brightness <= 255,
            "brightness out of range: {}",
            brightness
        );
    }

    #[test]
    fn test_tick_glow_y_is_zero() {
        let result = tick(500.0, 2.0, 1, 5.0);
        let y_bits = (result >> 32) as u32;
        let y = f32::from_bits(y_bits);
        assert!((y - 0.0).abs() < 0.001, "glow y should be 0, got {}", y);
    }

    #[test]
    fn test_tick_pulse_y_is_negative_or_zero() {
        let result = tick(750.0, 1.0, 2, 2.0);
        let y_bits = (result >> 32) as u32;
        let y = f32::from_bits(y_bits);
        assert!(y <= 0.001, "pulse y should be <= 0, got {}", y);
    }

    #[test]
    fn test_sin_approximation() {
        let pi = 3.14159265;
        assert!((sin(0.0)).abs() < 0.01);
        assert!((sin(pi / 2.0) - 1.0).abs() < 0.01);
        assert!((sin(pi)).abs() < 0.05);
        assert!((sin(-pi / 2.0) + 1.0).abs() < 0.01);
    }

    #[test]
    fn test_different_speeds_produce_different_results() {
        let r1 = tick(1000.0, 0.0, 0, 1.0);
        let r2 = tick(1000.0, 0.0, 0, 5.0);
        assert_ne!(r1, r2, "different speeds should produce different results");
    }
}
