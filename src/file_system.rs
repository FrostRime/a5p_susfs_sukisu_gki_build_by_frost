use std::{fs, io, os};
use std::path::Path;

use log::*;

pub fn create_symlink(original: &Path, link: &Path) -> io::Result<()> {
    #[cfg(target_family = "unix")]
    os::unix::fs::symlink(original, link)?;

    #[cfg(target_family = "windows")]
    {
        let metadata = fs::metadata(original)?;
        if metadata.is_dir() {
            os::windows::fs::symlink_dir(original, link)?;
        } else {
            os::windows::fs::symlink_file(original, link)?;
        }
    }
    Ok(())
}

// 移动单个文件
fn move_file(src: &Path, dest: &Path) -> io::Result<()> {
    // 优先尝试原子性重命名操作
    if let Err(rename_err) = fs::rename(src, dest) {
        // 仅在重命名失败时使用复制+删除策略
        fs::copy(src, dest)?;
        fs::remove_file(src)?;
        // 记录非常规移动方式（用于调试跟踪）
        debug!("回退到复制+删除方式移动文件: {:?}", rename_err);
    }
    info!("移动文件成功: {} → {}", src.display(), dest.display());
    Ok(())
}

pub fn move_fs(src: &Path, dest: &Path) -> io::Result<()> {
    if src.is_file() {
        // 智能构建目标路径
        let dest_path = if dest.is_dir() {
            dest.join(
                src.file_name()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "源路径不包含有效文件名"))?,
            )
        } else {
            dest.to_path_buf()
        };

        // 确保目标目录结构存在
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        return move_file(src, &dest_path);
    }

    // 处理目录移动
    fs::create_dir_all(dest)?; // 预先创建目标目录

    // 递归移动子项
    for entry in src.read_dir()? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_entry_path = dest.join(entry.file_name());
        move_fs(&entry_path, &dest_entry_path)?;
    }

    // 删除已清空的源目录
    fs::remove_dir(src)?;
    info!("移动目录成功: {} → {}", src.display(), dest.display());
    Ok(())
}