(module
  (import "abi" "exit" (func $exit (param i64) (result i64)))
  (func $main
    i64.const 42
    (drop (call $exit))
  )
  (start $main)
)
