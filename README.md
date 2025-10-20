# Name Exchanger Library (Rust)

A cross-platform library for atomically exchanging names of two files or directories, written in Rust.

一个使用Rust编写的跨平台库，用于原子性地交换两个文件或目录的名称。

## Overview 概述

This library provides safe and atomic file/directory name exchanges through exposed C-compatible interfaces. It can be called from other languages like C/C++, Python, etc.

该库通过暴露的 C 兼容接口提供安全且原子性的文件/目录名称交换功能。可以从C/C++、Python等其他语言调用。

## Usage 使用方法

### C Interface

```c
#include <stdint.h>

/// Exchange names of two files or directories
/// 
/// @param path1 - First file or directory path
/// @param path2 - Second file or directory path
/// @return 0 for success, non-zero values for errors:
///         0 - Success
///         1 - File does not exist
///         2 - Permission denied
///         3 - Target file already exists
///       255 - Unknown error
int32_t exchange(const char* path1, const char* path2);
```

### Rust Interface

```rust
use std::path::Path;

/// Exchange names of two files or directories
/// 
/// # Arguments
/// * `path1` - First file or directory path
/// * `path2` - Second file or directory path
/// 
/// # Returns
/// * `Ok(())` - Success
/// * `Err(RenameError)` - Error 
/// 
/// ## `RenameError` enum
/// ```rust
/// pub enum RenameError {
///    PermissionDenied,
///    AlreadyExists,
///    NotExists,
///    Unknown(String),
/// }
/// ```
fn exchange_rs(path1: &Path, path2: &Path) -> Result<(), RenameError>;
```

## Example 示例

Ref to 参考

1. [name_exchanger](https://github.com/Mikachu2333/name_exchanger)

2. [rs-NameExchanger](https://github.com/Mikachu2333/rs-NameExchanger)
