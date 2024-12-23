use std::{
    env,
    os::windows::process::CommandExt,
    process::{Command, Output},
};

use log::{error, info};

use crate::sys::global::{to_global_result, GlobalError, GlobalResult};

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// @Author: DengLibin
/// @Date: Create in 2024-08-05 10:27:01
/// @Description: cmd

pub fn check_output(output: Output) -> GlobalResult<()> {
    let os = env::consts::OS.to_lowercase();
    //info!("操作系统:{}", os);
    let mut decoded_string = String::from_utf8_lossy(&output.stdout).into_owned();
    if os == "windows" {
        // info!("windows输出:{}", decoded_string);
        // 使用 GBK 编码将字节转换为  字符串
        decoded_string = to_global_result(rust_common::file_util::get_gbk_str(&output.stdout))?;
    }
    // 打印标准输出
    info!("cmd输出: {}", decoded_string);

    // 打印标准错误
    if !output.stderr.is_empty() {
        decoded_string = String::from_utf8_lossy(&output.stderr).into_owned();
        if os == "windows" {
            decoded_string = to_global_result(rust_common::file_util::get_gbk_str(&output.stderr))?;
        }
        error!("cmd错误: {}", decoded_string);
        return Err(GlobalError::new(decoded_string));
    }
    Ok(())
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-17 09:34:51
/// @Description: 打开文件夹并选中文件
pub fn open_folder_and_select_file(file_path: &str) {
    if cfg!(target_os = "windows") {
        let path = file_path.replace("/", "\\");
        // Windows: 使用 explorer 并定位文件
        Command::new("explorer")
            .args(["/select,", &path])
            .creation_flags(CREATE_NO_WINDOW) // 隐藏控制台窗口
            .spawn()
            .expect("Failed to open file in explorer");
    } else if cfg!(target_os = "macos") {
        // macOS: 使用 open 命令
        Command::new("open")
            .args(["-R", file_path])
            .creation_flags(CREATE_NO_WINDOW) // 隐藏控制台窗口
            .spawn()
            .expect("Failed to open file in Finder");
    } else if cfg!(target_os = "linux") {
        // Linux: 使用 xdg-open 打开文件夹
        let folder_path = std::path::Path::new(file_path)
            .parent()
            .expect("Failed to get folder path");
        Command::new("xdg-open")
            .arg(folder_path)
            .spawn()
            .expect("Failed to open folder in file manager");
    } else {
        eprintln!("Unsupported operating system");
    }
}
