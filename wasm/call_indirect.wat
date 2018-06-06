(module
  (import "abi" "exit" (func $output (param i64) (result i64)))
  (type $test_func_type (func))
  (table 1 anyfunc)
  (elem (i32.const 0)
    $test_func
  )
  (func $test_func
    i64.const 41
    (drop (call $output))
  )
  (func $main
    i32.const 0
    (call_indirect (type $test_func_type))
  )
  (start $main)
)
