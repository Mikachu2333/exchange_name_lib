use std::{io, path::PathBuf};

/// 用于生成临时文件名的唯一标识符
/// 这个GUID用于创建临时文件名，确保不会与现有文件冲突
pub const GUID: &str = "E642A71D305C343884C";
pub const DEBUG_MODE: bool = cfg!(debug_assertions);

/// 存储文件或目录的元数据信息
///
/// 包含文件或目录的名称、扩展名和父目录路径
#[derive(Default, Debug, Clone)]
pub struct MetadataCollection {
    /// 文件名或目录名（不包含扩展名）
    pub name: String,
    /// 文件扩展名（包含前导点"."），目录为空字符串
    pub ext: String,
    /// 父目录的路径
    pub parent_dir: PathBuf,
}

/// 存储文件重命名所需的路径信息
///
/// 包含原始路径、新路径和临时过渡路径
#[derive(Default, Debug, Clone)]
pub struct PrepareName {
    /// 文件或目录的原始路径
    pub original_path: PathBuf,
    /// 重命名后的目标路径
    pub new_path: PathBuf,
    /// 重命名过程中使用的临时路径
    pub pre_path: PathBuf,
}

/// 存储文件完整信息的结构体
///
/// 包含文件存在状态、类型信息和重命名所需的路径数据
#[derive(Default, Debug, Clone)]
pub struct FileInfos {
    /// 文件或目录是否存在
    pub is_exist: bool,
    /// 是文件(true)还是目录(false)
    pub is_file: bool,
    /// 文件元数据信息（名称、扩展名和父目录）
    pub packed_info: MetadataCollection,
    /// 重命名所需的路径信息
    pub exchange: PrepareName,
}

/// 处理两个路径的结构体，用于路径检查和操作
#[derive(Default, Debug, Clone)]
pub struct GetPathInfo {
    /// 第一个文件或目录路径
    pub path1: PathBuf,
    /// 第二个文件或目录路径
    pub path2: PathBuf,
}

/// 文件交换名称的主要结构体
///
/// 包含两个文件的完整信息，用于执行重命名操作
pub struct NameExchange {
    /// 第一个文件的完整信息
    pub f1: FileInfos,
    /// 第二个文件的完整信息
    pub f2: FileInfos,
}

/// 重命名流程内部使用的错误类型
#[derive(Debug, Clone)]
pub enum RenameError {
    PermissionDenied,
    AlreadyExists,
    NotExists,
    Unknown(String),
}
impl RenameError {
    /// 将内部错误映射为最终返回码
    pub fn to_code(&self) -> i32 {
        match self {
            Self::NotExists => 1,
            Self::PermissionDenied => 2,
            Self::AlreadyExists => 3,
            Self::Unknown(_) => 255,
        }
    }
}
impl std::fmt::Display for RenameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::AlreadyExists => write!(f, "File already exists"),
            Self::NotExists => write!(f, "File does not exist"),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}
impl From<io::Error> for RenameError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::NotFound => RenameError::NotExists,
            io::ErrorKind::PermissionDenied => RenameError::PermissionDenied,
            io::ErrorKind::AlreadyExists => RenameError::AlreadyExists,
            _ => RenameError::Unknown(value.to_string()),
        }
    }
}
