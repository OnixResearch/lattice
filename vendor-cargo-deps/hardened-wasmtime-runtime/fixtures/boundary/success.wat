(module
  (memory (export "memory") 1)
  (func (export "run") (param $ptr i32) (param $len i32) (result i32)
    (i32.store8 (local.get $ptr) (i32.const 79)) ;; 'O'
    (i32.store8 (i32.add (local.get $ptr) (i32.const 1)) (i32.const 75)) ;; 'K'
    (i32.const 2)))
