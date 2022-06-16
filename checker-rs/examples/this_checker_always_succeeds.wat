(module
  (import "checker" "succeed" (func $succeed (param i32 i32)))
    (func (export "check")
      i32.const 4   ;; ptr
      i32.const 27  ;; len
      call $succeed)
  (memory (export "memory") 2) 
  (data (i32.const 4) "This checker always succeed")
)
