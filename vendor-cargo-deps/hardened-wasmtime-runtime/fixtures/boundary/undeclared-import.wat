(module
  (import "env" "amb" (func $amb))
  (memory (export "memory") 1)
  (func (export "run") (param $ptr i32) (param $len i32) (result i32)
    (call $amb)
    (i32.const 0)))
