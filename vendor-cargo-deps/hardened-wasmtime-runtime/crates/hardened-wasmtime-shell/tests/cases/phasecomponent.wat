(component
  (import "host-step" (func $host))
  (core func $lower (canon lower (func $host)))
  (core instance $host (export "step" (func $lower)))
  (core module $m
    (import "host" "step" (func $step))
    (memory (export "memory") 1)
    (func $start {start})
    (start $start)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      i32.const 1024)
    (func (export "run") (param i32 i32) (result i32)
      {action}
      call $step
      i32.const 0 i32.const 0 i32.store
      i32.const 4 i32.const 32 i32.store
      i32.const 8 i32.const 2 i32.store
      i32.const 32 i32.const 19279 i32.store16
      i32.const 0))
  (core instance $i (instantiate $m (with "host" (instance $host))))
  (alias core export $i "memory" (core memory $memory))
  (alias core export $i "realloc" (core func $realloc))
  (alias core export $i "run" (core func $run))
  (func (export "invoke") (param "input" (list u8))
    (result (result (list u8) (error string)))
    (canon lift (core func $run) (memory $memory) (realloc $realloc))))
