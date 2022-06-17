(module
    ;; Import the required fd_write WASI function which will write the given io vectors to stdout
    ;; The function signature for fd_write is:
    ;; (File Descriptor, *iovs, iovs_len, nwritten) -> Returns number of bytes written
    (import "wasi_unstable" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))

    (memory 2)
    (export "memory" (memory 0))


    (func (export "check")
        ;; Creating a new io vector within linear memory
        (i32.store (i32.const 0) (i32.const 65538))  ;; first dataset is always at 65536, the first item is the size of the dataset, the rest is the dataset in csv format
        (i32.store (i32.const 4) (i32.const 20))  ;; iov.iov_len - The length of the csv, 17 = length of the csv; + 1 for the size

        (call $fd_write
            (i32.const 1) ;; file_descriptor - 1 for stdout
            (i32.const 0) ;; *iovs - The pointer to the iov array, which is stored at memory location 0
            (i32.const 1) ;; iovs_len - We're printing 1 string stored in an iov - so one.
            (i32.const 20) ;; nwritten - A place in memory to store the number of bytes written
        )
        drop ;; Discard the number of bytes written from the top of the stack
    )
)
