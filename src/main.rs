use dialogue_macro::Asker;
use diff_match_patch_rs::{dmp, Compat};
use file_system::*;
use log::*;
use regex::Regex;
use repo::*;
use std::{
    fs, io::Write, path::{Path, PathBuf}
};
use which::which;

mod file_system;
mod repo;
mod patch;
mod sukisu;

#[derive(Asker, Debug)]
struct Info {
    #[input(
        prompt = "请输入自定义内核名:",
        default = "android15-8-g013ec21bba94-abogki383916444"
    )]
    kernel_name: String,

    #[input(prompt = "请输入输出路径:", default = "output_dir/")]
    output_dir: String,

    #[select(prompt = "SukiSU更新频道:", options = ["main", "dev"], default = 1)]
    sukisu_branch: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe { std::env::set_var("RUST_LOG", "info") };
    env_logger::init();
    info!("欢迎使用此工具!");

    info!("检测系统环境...");
    match which("bazel") {
        Err(which::Error::CannotFindBinaryPath) => {
            error!("Bazel 未安装,请安装后重试!");
            return Err("Bazel 未安装".into());
        }
        Err(e) => {
            error!("Bazel 检测失败: {:?}", e);
            return Err(Box::new(e));
        }
        Ok(_path) => {
            info!("已找到 Bazel");
        }
    }

    info!("初始化环境...");

    let tempdir = tempfile::tempdir()?;
    let tempdir_path = Path::new("tempdir");
    fs::create_dir_all(tempdir_path);

    let tempdir_work_dir = setup_tempdir_work_dir(&tempdir_path);
    let tempdir_work_kernel_platform_dir = tempdir_work_dir.join(Path::new("kernel_platform"));
    let tempdir_work_kernel_platform_common_dir =
        tempdir_work_kernel_platform_dir.join(Path::new("common"));
    let tempdir_work_kernel_platform_fs_dir =
        tempdir_work_kernel_platform_dir.join(Path::new("fs"));
    let tempdir_work_kernel_platform_include_dir =
        tempdir_work_kernel_platform_dir.join(Path::new("include"));
    let tempdir_work_kernel_platform_include_linux_dir =
        tempdir_work_kernel_platform_include_dir.join(Path::new("linux"));
    let tempdir_work_kernel_platform_common_lib_dir =
        tempdir_work_kernel_platform_common_dir.join(Path::new("lib"));
    let tempdir_work_kernel_platform_common_crypto_dir =
        tempdir_work_kernel_platform_common_dir.join(Path::new("crypto"));
    let tempdir_work_kernel_platform_common_patch_file = tempdir_work_kernel_platform_common_dir
        .join(Path::new("50_add_susfs_in_gki-android15-6.6.patch"));

    info!("已找到缓存文件夹: {}", tempdir_path.to_str().unwrap());

    let info = Info::asker()
        .kernel_name()
        .output_dir()
        .sukisu_branch()
        .finish();

    setup_output_dir(&info.output_dir);

    info!("拉取Oneplus Ace5 Pro内核清单...");

    clone_retry(
        &tempdir_work_dir.join(Path::new("oneplus-kernel-manifest")),
        "oneplus/sm8750",
        "https://github.com/OnePlusOSS/kernel_manifest.git",
        10,
        true,
    );

    replace_makefile_version(&tempdir_work_kernel_platform_dir.join(Path::new("KernelSU/kernel/Makefile")), sukisu::setup_sukisu(&tempdir_work_kernel_platform_dir, &info.sukisu_branch)?)?;

    let tempdir_work_susfs_dir = tempdir_work_dir.join(Path::new("susfs"));
    let susfs_branch = "gki-android15-6.6";
    info!("拉取 susfs...");

    clone_retry(
        &tempdir_work_susfs_dir,
        &susfs_branch,
        "https://gitlab.com/simonpunk/susfs4ksu.git",
        10,
        true,
    );

    if !Path::new(&tempdir_work_dir).exists() {
        fs::create_dir_all(&tempdir_work_dir)?;
    }

    let kernel_patches_dir =
        tempdir_work_susfs_dir.join(Path::new("kernel_patches"));
    let kernel_patches_tempdir_work_kernel_platform_fs_dir =
        kernel_patches_dir.join(Path::new("fs"));
    let kernel_patches_include_linux = kernel_patches_dir.join(Path::new("include/linux"));
    let kernel_patches_tempdir_work_kernel_platform_common_patch_file =
        kernel_patches_dir.join(Path::new("50_add_susfs_in_gki-android15-6.6.patch"));

    move_fs(
        &kernel_patches_tempdir_work_kernel_platform_fs_dir,
        &tempdir_work_kernel_platform_fs_dir,
    )?;
    move_fs(
        &kernel_patches_include_linux,
        &tempdir_work_kernel_platform_include_linux_dir,
    )?;
    move_fs(
        &kernel_patches_tempdir_work_kernel_platform_common_patch_file,
        &tempdir_work_kernel_platform_common_patch_file,
    )?;

    info!("susfs 拉取完成!");

    info!("拉取 SukiSU Patch...");

    clone_retry(
        &tempdir_work_dir.join(Path::new("sukisu-patch")),
        "main",
        "https://github.com/ExmikoN/SukiSU_patch.git",
        10,
        true,
    );

    info!("SukiSU Patch 拉取完成!");

    info!("应用 LZ4K 补丁...");

    let sukisu_patch_dir = tempdir_work_dir.join(Path::new("sukisu-patch"));
    let sukisu_patch_other_dir = sukisu_patch_dir.join(Path::new("other"));
    let sukisu_patch_other_lz4k_dir = sukisu_patch_other_dir.join(Path::new("lz4k"));
    let sukisu_patch_other_lz4k_tempdir_work_kernel_platform_include_dir =
        sukisu_patch_other_lz4k_dir.join(Path::new("include"));
    let sukisu_patch_other_lz4k_include_tempdir_work_kernel_platform_include_linux_dir =
        sukisu_patch_other_lz4k_tempdir_work_kernel_platform_include_dir.join(Path::new("linux"));
    let sukisu_patch_other_lz4k_tempdir_work_kernel_platform_common_lib_dir =
        sukisu_patch_other_lz4k_dir.join(Path::new("lib"));
    let sukisu_patch_other_lz4k_tempdir_work_kernel_platform_common_crypto_dir =
        sukisu_patch_other_lz4k_dir.join(Path::new("crypto"));

    move_fs(
        &sukisu_patch_other_lz4k_include_tempdir_work_kernel_platform_include_linux_dir,
        &tempdir_work_kernel_platform_include_linux_dir,
    )?;
    move_fs(
        &sukisu_patch_other_lz4k_tempdir_work_kernel_platform_common_lib_dir,
        &tempdir_work_kernel_platform_common_lib_dir,
    )?;
    move_fs(
        &sukisu_patch_other_lz4k_tempdir_work_kernel_platform_common_crypto_dir,
        &tempdir_work_kernel_platform_common_crypto_dir,
    )?;

    patch::apply_directory_patch(&tempdir_work_kernel_platform_common_dir, &kernel_patches_tempdir_work_kernel_platform_common_patch_file);

    info!("应用完补丁");

    move_fs(&tempdir_work_dir, Path::new("work"));

    Ok(())
}

fn setup_tempdir_work_dir(tempdir: &Path) -> PathBuf {
    let tempdir_work_dir = tempdir.join(Path::new("work"));
    if !&tempdir_work_dir.exists() {
        fs::create_dir_all(&tempdir_work_dir).unwrap();
    }
    tempdir_work_dir
}

fn setup_output_dir(output_dir: &str) {
    let output_dir = Path::new(output_dir);
    if output_dir.exists() {
        if !output_dir.is_dir() {
            panic!("输出路径不是文件夹");
        }
        info!("输出文件夹已存在");
    } else {
        fs::create_dir_all(output_dir).expect("创建输出文件夹失败");
    }
}

// 实现sed -i "s/DKSU_VERSION=12800/DKSU_VERSION=${KSU_VERSION}/" kernel/Makefile
pub fn replace_makefile_version(makefile_path: &Path, ksu_version: usize) -> Result<(), Box<dyn std::error::Error>> {
    // 构建正则表达式模式
    let pattern = Regex::new(r"^DKSU_VERSION=12800$")?; // 精确匹配整行
    let replacement = format!("DKSU_VERSION={}", ksu_version);

    // 读取文件内容
    let content = fs::read_to_string(makefile_path)?;
    
    // 逐行处理并替换
    let modified_content = content
        .lines()
        .map(|line| {
            if pattern.is_match(line) {
                replacement.as_str() // 替换匹配行
            } else {
                line // 保留其他行
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 写回文件（实现-i参数效果）
    let mut file = fs::File::create(makefile_path)?;
    file.write_all(modified_content.as_bytes())?;

    Ok(())
}