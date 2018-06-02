(module
  (import "abi" "exit" (func $exit (param i64) (result i64)))
  (import "abi" "write_i64" (func $write_i64 (param i32) (param i64) (result i64)))
  (import "abi" "assert_eq" (func $assert_eq (param i64) (param i64) (result i64)))
  (export "memory" (memory $0))
  (memory $0 0)
  (func $main
    i32.const 1
    grow_memory
    i64.extend_u/i32
    (drop (call $exit))
  )
  (start $main)
)
