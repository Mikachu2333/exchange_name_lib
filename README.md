# Name Exchanger Library

跨平台 Rust 库，用于交换两个文件、目录或符号链接的名称，提供 Rust 与 C ABI。

> 每一步重命名在同一文件系统内是原子的，但完整交换由三步组成，不是崩溃安全的文件系统事务。进程崩溃、断电或其他进程同时修改相关路径时，仍可能留下中间状态。

## Rust API

```rust
use exchange_name_lib::exchange_rs;
use std::path::Path;

exchange_rs(Path::new("alpha.txt"), Path::new("beta.log"), false)?;
# Ok::<(), exchange_name_lib::RenameError>(())
```

`preserve_ext = true` 时，普通文件保留各自扩展名，仅交换文件名主体。目录和符号链接交换完整名称。

## C API

使用仓库中的 [`exchange_name_lib.h`](exchange_name_lib.h)。路径必须是 UTF-8；推荐使用带显式长度的 `exchange_n`。旧接口 `exchange` 要求指针指向 NUL 结尾字符串，库无法验证缓冲区边界。

```c
#include "exchange_name_lib.h"

int32_t result = exchange("alpha.txt", "beta.log", 0);
```

错误码：

|  值 | 含义                               |
| --: | ---------------------------------- |
|   0 | 成功                               |
|   1 | 路径不存在                         |
|   2 | 权限不足或只读文件系统             |
|   3 | 目标已存在                         |
|   4 | 两个路径指向同一项                 |
|   5 | 路径、UTF-8 或布尔参数无效         |
|   6 | 不支持的特殊文件类型               |
|   7 | 操作与回滚均失败，可能需要人工恢复 |
| 255 | 未知错误或捕获到 panic             |

## 行为与限制

- 不裁剪路径空白，也不解析 shell 引号。
- 不解引用最终路径组件的符号链接。
- 拒绝交换互为祖先与后代的目录，避免中途路径失效。
- 进程内调用串行执行，以避免本库线程之间互相干扰；这不能锁定其他进程。
- 两个条目及临时目录必须位于允许重命名的同一文件系统范围内。
- Unix Rust API 支持非 UTF-8 路径；C API 仅接受 UTF-8。
- 库不包含 GUI，因此 GUI 布局检查不适用。

## 构建与验证

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo package
```

Windows MSVC 双架构构建：

```powershell
pwsh -File ./build.ps1
```
