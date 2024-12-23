//! @Author: DengLibin
//! @Date: Create in 2024-07-25 17:47:15
//! @Description: 7z 压缩 解压

use std::{os::windows::process::CommandExt, process::Command};

use crate::sys::global::{to_global_result, GlobalResult};

use super::cmd::check_output;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// @Author: DengLibin
/// @Date: Create in 2024-07-25 18:00:16
/// @Description: 解压文件
/// @param seven_z_path: 7z 压缩解压工具路径
/// @param file_path: 待解压文件路径
/// @param output_dir: 解压输出目录
pub fn decompress_file(seven_z_path: &str, file_path: &str, output_dir: &str) -> GlobalResult<()> {
    // 执行命令并获取输出
    let output = Command::new(seven_z_path)
        .arg("x")
        .arg(file_path)
        .arg(format!("-o{}", output_dir))
        .creation_flags(CREATE_NO_WINDOW) // 隐藏控制台窗口
        .output();

    let output = to_global_result(output)?;

    check_output(output)
}

/// @Author: DengLibin
/// @Date: Create in 2024-07-26 09:32:09
/// @Description: 文件夹压缩为zip包
/// @param seven_z_path: 7z 压缩解压工具路径
/// @param dir: 待压缩文件夹路径
/// @param out_file: 压缩输出文件路径
pub fn compress_dir_to_zip(seven_z_path: &str, dir: &str, out_file: &str) -> GlobalResult<()> {
    // 执行命令并获取输出
    let output = Command::new(seven_z_path)
        .arg("a")
        .arg("-mcu")
        .arg("-tzip")
        .arg(out_file)
        .arg(dir)
        .creation_flags(CREATE_NO_WINDOW) // 隐藏控制台窗口
        .output();

    let output = to_global_result(output)?;

    check_output(output)
}
