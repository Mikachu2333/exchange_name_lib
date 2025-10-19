use std::path::{Path, PathBuf};

use crate::types::*;

/// 改名逻辑主体整合
impl NameExchange {
    /// 用于初始化储存所有信息的结构体
    ///
    /// 创建一个新的 NameExchange 实例，其中包含两个默认初始化的 FileInfos
    pub fn new() -> NameExchange {
        NameExchange {
            f1: FileInfos {
                ..Default::default()
            },
            f2: FileInfos {
                ..Default::default()
            },
        }
    }

    /// 获取临时文件名与改后文件名
    ///
    /// 根据目录路径、文件名和扩展名生成临时文件路径和最终文件路径
    ///
    /// ### 参数
    /// * `dir` - 文件所在的目录路径
    /// * `other_name` - 目标文件名（不含扩展名）
    /// * `ext` - 文件扩展名（包含前导点"."）
    ///
    /// ### 返回值
    /// 返回元组 `(临时文件路径, 最终文件路径)`
    pub fn make_name(dir: &Path, other_name: impl ToString, ext: impl ToString) -> (PathBuf, PathBuf) {
        let other_name = other_name.to_string();
        let ext = ext.to_string();
        let mut temp_path = dir.to_path_buf();
        let mut final_path = dir.to_path_buf();

        // 任意长字符串用作区分
        let mut temp_name = crate::types::GUID.to_string();
        temp_name.push_str(&ext);
        temp_path.push(temp_name);

        let final_component = if ext.is_empty() {
            other_name
        } else {
            format!("{}{}", other_name, ext)
        };

        if !final_component.is_empty() {
            final_path.push(final_component);
        }

        (temp_path, final_path)
    }

    /// 改名具体执行部分
    ///
    /// 根据文件类型和嵌套关系执行重命名操作
    ///
    /// ### 参数
    /// * `is_nested` - 是否是嵌套关系（如父子目录）
    /// * `file1_first` - 是否先重命名第一个文件
    ///
    /// ### 返回值
    /// 返回 `Ok(())` 表示成功，`Err(RenameError)` 表示对应的失败原因
    pub fn rename_each(&self, is_nested: bool, file1_first: bool) -> Result<(), RenameError> {
        // 根据重命名顺序准备路径变量
        let mut path1 = self.f2.exchange.original_path.clone();
        let mut final_name1 = self.f2.exchange.new_path.clone();
        let mut path2 = self.f1.exchange.original_path.clone();
        let mut final_name2 = self.f1.exchange.new_path.clone();
        let mut tmp_name2 = self.f1.exchange.pre_path.clone();
        if file1_first {
            path1 = self.f1.exchange.original_path.clone();
            final_name1 = self.f1.exchange.new_path.clone();
            path2 = self.f2.exchange.original_path.clone();
            final_name2 = self.f2.exchange.new_path.clone();
            tmp_name2 = self.f2.exchange.pre_path.clone();
        }

        //1 first
        if is_nested {
            // 如果存在嵌套关系（父子目录或文件），直接按顺序重命名
            // 不使用临时文件，因为嵌套关系下使用临时文件可能引起路径问题
            Self::handle_rename(&path1, &final_name1)?;
            Self::handle_rename(&path2, &final_name2)?;
            Ok(())
        } else {
            // 不存在嵌套关系：使用临时文件进行安全交换
            // 1. 将第二个文件重命名为临时文件
            // 2. 将第一个文件重命名为最终名称
            // 3. 将临时文件重命名为最终名称
            Self::handle_rename(&path2, &tmp_name2)?;
            Self::handle_rename(&path1, &final_name1)?;
            Self::handle_rename(&tmp_name2, &final_name2)?;
            Ok(())
        }
    }

    /// 处理单个重命名操作并处理可能的错误
    ///
    /// ### 参数
    /// * `from` - 原始文件路径
    /// * `to` - 目标文件路径
    ///
    /// ### 返回值
    /// 返回 `Ok(())` 表示成功，`Err(RenameError)` 表示具体错误
    fn handle_rename(from: &Path, to: &Path) -> Result<(), RenameError> {
        match std::fs::rename(from, to) {
            Ok(_) => Ok(()),
            Err(e) => Err(RenameError::from(e)),
        }
    }
}
