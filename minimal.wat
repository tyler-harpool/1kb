(module
  (func (export "s") (param f32) (result f32)
    local.get 0
    f32.const 1.0
    local.get 0
    local.get 0
    f32.mul
    local.tee 0
    f32.const 0.16666667
    local.get 0
    local.get 0
    f32.const -5040.0
    f32.div
    f32.const 0.008333334
    f32.add
    f32.mul
    f32.sub
    f32.mul
    f32.sub
    f32.mul
  )
)
