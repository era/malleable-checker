(module
(import "host" "hello" (func $host_hello))  
  (func (export "check")
    call $host_hello
  )
)
