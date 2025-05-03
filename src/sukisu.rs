use std::path::Path;

use log::*;

use crate::{repo, file_system::*};

//设置SukiSu-Ultra
pub fn setup_sukisu(kernel_platform_dir: &Path, branch_name: &str) -> Result<usize, git2::Error> {
    let sukisu_dir = kernel_platform_dir.join("KernelSU");
    let sukisu_driver_dir = kernel_platform_dir.join("common/drivers");
    info!("开始初始化 SukiSU-Ultra 仓库");

    let ksu_version = repo::get_commit_count(&repo::clone_retry(&sukisu_dir, &branch_name, "https://github.com/ShirkNeko/SukiSU-Ultra", 10, false), branch_name)? + 10606;

    info!("当前 SukiSU 版本: {}", ksu_version);

    info!("初始化完成!");

    // 创建符号链接
    info!("正在初始化符号链接...");
    let symlink_target = &sukisu_driver_dir.join("kernelsu");
    if let Some(parent) = symlink_target.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("创建父目录失败");
        }
    }

    if let Err(err) = create_symlink(Path::new(&sukisu_dir), symlink_target) {
        return Err(git2::Error::from_str(&format!("创建符号链接失败:{}", err)));
    }

    info!("符号链接初始化完成!");

    Ok(ksu_version)
}
