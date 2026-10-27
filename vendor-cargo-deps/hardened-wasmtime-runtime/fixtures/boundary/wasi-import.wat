(module
  (import "wasi_snapshot_preview1" "fd_read" (func $fd_read))
  (memory (export "memory") 1)
  (func (export "run") (param $ptr i32) (param $len i32) (result i32)
    (call $fd_read)
    (i32.const 0)))
