use std::{error::Error, path::Path};

use diff_match_patch_rs::DiffMatchPatch;

pub fn apply_patch(source: &Path, patch_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let dmp = DiffMatchPatch::new();
    let source_text = std::fs::read_to_string(source)?;
    let patch_text = std::fs::read_to_string(patch_path)?;

    // 添加显式类型参数声明
    let patches = dmp
        .patch_from_text::<char>(&patch_text)
        .map_err(|e| format!("补丁解析失败: {:?}", e))?;

    match dmp.patch_apply(&patches, &source_text) {
        Ok((result, _)) => {
            std::fs::write(source, &result)?;
            Ok(result)
        }
        Err(e) => Err(format!("补丁应用失败: {:?}", e).into()),
    }
}

pub fn apply_directory_patch(dir: &Path, patch_path: &Path) -> Result<(), Box<dyn Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            apply_patch(&entry.path(), patch_path)?;
        }
    }
    Ok(())
}
