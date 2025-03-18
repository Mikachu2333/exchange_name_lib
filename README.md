直接调用暴露的 `exchange()` 函数即可。
Use the `exchange()` function directly.

```rust
/// 0 => Success，1 => No Exist
///
/// 2 => Permission Denied，3 => New File Already Exists
///
/// 255 => UNKNOWN ERROR
pub extern "C" fn exchange(path1: *const c_char, path2: *const c_char) -> i32{
    ...
}
```

