(module
  (import "abi" "exit" (func $exit (param i64)))
  (func $main
    i64.const 0
    (call $exit)
  )
  (start $main)
)