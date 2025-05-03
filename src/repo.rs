use git2::*;
use indicatif::*;
use log::{error, warn};
use std::{cell::*, path::Path};

/// 克隆指定分支的Git仓库到目标目录
///
/// # 参数
/// - `target_dir`: 目标目录路径（将在此目录初始化仓库）
/// - `branch_name`: 需要克隆的远程分支名称
/// - `url`: Git仓库远程URL
///
/// # 返回值
/// 返回`Result<(), git2::Error>`表示操作结果
///
/// # 实现流程
/// 1. 初始化本地Git仓库
/// 2. 配置远程仓库和进度回调
/// 3. 获取指定分支数据
/// 4. 创建本地分支并强制覆盖（当force=true时）
/// 5. 检出工作区文件
pub fn clone(target_dir: &Path, branch_name: &str, url: &str, min_clone: bool) -> Result<Repository, git2::Error> {
    // 初始化目标仓库目录
    let repo = git2::Repository::init(&target_dir)?;

    // 远程清理逻辑
    if repo.find_remote("origin").is_ok() {
        repo.remote_delete("origin")?;
    }

    {
        // 配置远程仓库（自动覆盖已存在的origin远程）
        let mut remote = repo.remote("origin", url)?;

        // 创建带进度条的回调函数
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        let progress_bar = RefCell::new(None::<ProgressBar>);
        callbacks.transfer_progress(move |progress| {
            // 进度条初始化与更新逻辑
            let mut bar = progress_bar.borrow_mut();
            if bar.is_none() {
                *bar = Some(ProgressBar::new(progress.total_objects() as u64));
            }
            if let Some(b) = bar.as_mut() {
                if progress.received_objects() == progress.total_objects() {
                    b.finish_with_message("\n");
                } else {
                    b.set_position(progress.received_objects() as u64);
                }
            }
            true
        });
        fetch_options.remote_callbacks(callbacks);

        if min_clone {
            fetch_options.depth(1);
        }

        // 获取远程分支数据（使用强制覆盖策略）
        let refspec = format!("{}:{}", branch_name, branch_name);
        remote.fetch(&[&refspec], Some(&mut fetch_options), None)?;

        // 处理分支引用
        let branch_ref = format!("refs/remotes/origin/{}", branch_name);
        let reference = repo.find_reference(&branch_ref)?;
        let annotated_commit = repo.reference_to_annotated_commit(&reference)?;
        let commit = repo.find_commit(annotated_commit.id())?;

        // 创建/覆盖本地分支（force=true表示强制覆盖已存在分支）
        repo.branch(branch_name, &commit, true)?;

        // 设置HEAD指向新分支并强制检出工作区
        repo.set_head(&format!("refs/heads/{}", branch_name))?;
        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.force();
        repo.checkout_head(Some(&mut checkout_builder))?;
    }

    Ok(repo)
}

pub fn clone_retry(target_dir: &Path, branch_name: &str, url: &str, retry_count: usize, min_clone: bool) -> Repository {
    for _ in 0..retry_count {
        match clone(target_dir, branch_name, url, min_clone) {
            Ok(repo) => return repo,
            Err(err) => warn!("克隆失败，剩余重试次数 {}: {}", retry_count - 1, err.message()),
        }
    }
    error!("拉取失败，请检查网络连接！\n并重新运行程序！");
    std::process::exit(1);
}

pub fn get_commit_count(repo: &Repository, branch_name: &str) -> Result<usize, git2::Error> {
    let reference = repo.find_reference(&format!("refs/heads/{}", branch_name))?;
    let commit = reference.peel_to_commit()?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push(commit.id())?;

    Ok(revwalk.count())
}
