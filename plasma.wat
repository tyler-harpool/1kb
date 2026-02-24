(module
  (func (export "f") (param i32 i32 i32) (result i32)
    ;; plasma effect: ((x * y) ^ t) & 0xFF
    ;; creates moiré interference pattern that shifts over time
    local.get 0    ;; x
    local.get 1    ;; y
    i32.mul        ;; x * y
    local.get 2    ;; t
    i32.xor        ;; (x * y) ^ t
    i32.const 255
    i32.and        ;; clamp to 0-255
  )
)
