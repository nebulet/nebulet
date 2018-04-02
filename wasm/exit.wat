(module
  (import "abi" "exit" (func $exit (param i64)))
  (func $main
    i64.const 42
    (call $exit)
  )
  (start $main)
)
