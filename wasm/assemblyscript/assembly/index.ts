// The entry file of your WebAssembly module.

class Result {kind: string; error: string}

export function assert_equal<T>(collection: Array<T>, expected: Array<T>, message: string): Result {
  return assert_true(collection == expected, message)
}

export function assert_empty<T>(collection: Array<T>, min_size: i32, message: string): Result {
  return assert_true(collection.length == 0, message)
}

export function assert_less_than<T>(collection: Array<T>, min_size: i32, message: string): Result {
  return assert_true(collection.length < min_size, message)
}

export function assert_greater_than<T>(collection: Array<T>, min_size: i32, message: string): Result {
  return assert_true(collection.length > min_size, message)
}

export function assert_false(expression: bool, message: string): Result {
  return assert_true(!expression, message)
}

export function assert_true(expression: bool, message: string): Result {
  if (!expression) {
    return {kind: "Failed", error: message}
  } else {
    return {kind: "Succeeded", error: ""}
  }
}

export function test(): string {
  return "a"
}