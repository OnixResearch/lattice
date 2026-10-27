(module
  (memory (export "memory") 1)
  (func (export "run") (param $ptr i32) (param $len i32) (result i32)
    unreachable))
