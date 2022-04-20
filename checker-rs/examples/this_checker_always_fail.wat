(module
  (import "checker" "fail" (func $fail (param i32 i32)))
    (func (export "check")
      i32.const 4   ;; ptr
      i32.const 24  ;; len
      call $fail)
  (memory (export "memory") 1)
  (data (i32.const 4) "This checker always fail")
)
