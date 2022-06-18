(module
  (import "checker" "succeed" (func $succeed (param i32 i32)))
  (import "checker" "datasets" (func $datasets (param i32 i32) (result i32 i32)))
    (func (export "check")

      i32.const 4   ;; ptr (pass "test" as param to $datasets)
      i32.const 4  ;; len
      call $datasets
      call $succeed) ;; passes the dataset to the succeed function
  (memory (export "memory") 2) 
  (data (i32.const 4) "test")
)
