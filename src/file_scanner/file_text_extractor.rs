//! @Author: DengLibin
//! @Date: Create in 2024-07-31 11:49:46
//! @Description: 文件内容（文本）提取器

use std::{path::Path, sync::Arc};

use encoding::{all::GBK, DecoderTrap, Encoding};
use extractous::Extractor;
use log::error;

use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    sync::mpsc::Sender,
};

use crate::sys::global::{to_global_result, GlobalResult};
use lazy_static::lazy_static;

use super::is_compress_file;

lazy_static! {

    //内容提取器， 最长 50_000
    static ref arc_extractor: Arc<Extractor> = Arc::new(Extractor::new().set_extract_string_max_length(500_000));
}
/// @Author: DengLibin
/// @Date: Create in 2024-07-31 11:51:55
/// @Description: 提取文件内容 docx, pptx, xlsx, pdf, 文件文件

#[derive(Debug)]
pub struct FileText {
    pub file_path: String, //文件路径
    pub success: bool,     //是否成功
    pub text: String,      //文件文本内容
    pub err: String,       //错误信息
}

// 文本抽取参数
pub struct TextExtractParam {
    pub file_path: String,                  //文件路径
    pub text_sender: Arc<Sender<FileText>>, //文件内容发送者
}

/// @Author: DengLibin
/// @Date: Create in 2024-07-31 11:53:00
/// @Description: 提取文件内容
pub async fn spawn_extract(param: TextExtractParam) {
    tokio::task::spawn(async {
        let r = do_extract_text(param).await;
        if r.is_err() {
            error!("提取文件内容异常:{}", r.err().unwrap());
        }
    });
}

/// @Author: DengLibin
/// @Date: Create in 2024-07-31 11:53:00
/// @Description: 提取文件内容
pub async fn spawn_extract_text(file_path: String, text_sender: Arc<Sender<FileText>>) {
    tokio::task::spawn(async {
        let r = do_extract_text(TextExtractParam {
            file_path,
            text_sender,
        })
        .await;
        if r.is_err() {
            error!("提取文件内容异常:{}", r.err().unwrap());
        }
    });
}

//提取文件内容
async fn do_extract_text(param: TextExtractParam) -> GlobalResult<()> {
    let TextExtractParam {
        file_path,
        text_sender,
    } = param;

    let metadata_r = fs::metadata(&file_path).await;
    let meta = to_global_result(metadata_r)?;
    //文件夹
    let text_r = if meta.is_dir() {
        Ok("".into())
    } else {
        let ext_op = get_ext_name(file_path.as_str());
        match ext_op {
            Some(ext) => match ext.as_str().to_lowercase().as_str() {
                //office 2007格式, 文档中的图片 走ocr
                "docx" | "xlsx" | "pptx" => extract_officex(file_path.clone(), ext).await,
                //图片格式
                "png" | "jpg" | "jpeg" | "bmp" | "gif" => Ok("".into()),

                //其他文件格式
                _ => extract_txt(file_path.as_str()).await,
            },
            None => Ok("".into()),
        }
    };

    match text_r {
        Ok(text) => {
            let file_text = FileText {
                file_path: file_path.clone(),
                success: true,
                text,
                err: "".into(),
            };
            to_global_result(text_sender.send(file_text).await)?;
        }
        Err(e) => {
            let file_text = FileText {
                file_path: file_path.clone(),
                success: false,
                text: "".into(),
                err: e.msg,
            };
            to_global_result(text_sender.send(file_text).await)?;
        }
    }
    Ok(())
}

/// @Author: DengLibin
/// @Date: Create in 2024-07-31 13:37:21
/// @Description: 读取utf8编码的txt文件
#[allow(dead_code)]
async fn extract_utf8_txt(file_path: &str) -> GlobalResult<String> {
    // 打开文件
    let mut file = to_global_result(File::open(file_path).await)?;

    // 创建一个字符串缓冲区来存储文件内容
    let mut contents = String::new();

    // 异步读取文件内容到缓冲区
    to_global_result(file.read_to_string(&mut contents).await)?;
    Ok(contents)
}

/// @Author: DengLibin
/// @Date: Create in 2024-07-31 13:44:53
/// @Description: 读取文本
async fn extract_txt(file_path: &str) -> GlobalResult<String> {
    //压缩文件
    if is_compress_file(file_path) {
        return Ok("".into());
    }
    
    //非文本文件
    if !is_text_file(file_path).await? {
        let file_path_1 = file_path.to_string();
        let r = tokio::task::spawn_blocking(move || {
            let r = arc_extractor.extract_file_to_string(&file_path_1);

            if let Ok(content) = r {
                return content.0;
            }
            return "".to_string();
        })
        .await;
        return to_global_result(r);
    }
    // 打开文件
    let mut file = to_global_result(File::open(file_path).await)?;

    // 创建一个字符串缓冲区来存储文件内容
    let mut buffer = Vec::new();

    // 异步读取文件内容到缓冲区
    to_global_result(file.read_to_end(&mut buffer).await)?;

    let is_utf8 = is_utf8(&buffer);
    if is_utf8 {
        return Ok(String::from_utf8_lossy(&buffer).into());
    }

    // 将字节缓冲区转换为指定编码的字符串
    let decoded_r = GBK.decode(&buffer, DecoderTrap::Strict);
    let content = match decoded_r {
        Ok(content) => content,
        Err(_) => "".into(),
    };

    Ok(content)
}

/// @Author: DengLibin
/// @Date: Create in 2024-07-31 14:58:27
/// @Description: 提取office 2007 格式文本
/// @param file_path 文件路径
/// @param ext_name 文件扩展名
/// @param pic_dir 图片保存目录
/// @return GlobalResult<String> 文本内容
async fn extract_officex(file_path: String, ext_name: String) -> GlobalResult<String> {
    let arc_file_path = Arc::new(file_path);
    let file_path = arc_file_path.clone();
    //阻塞任务 交给阻塞线程完成
    let r = tokio::task::spawn_blocking(move || {
        let r = match ext_name.as_str() {
            "docx" => {
                rust_common::office_text_extractor::extract_text_from_docx(file_path.as_str())
            }
            "xlsx" => {
                rust_common::office_text_extractor::extract_text_from_xlsx(file_path.as_str())
            }
            "pptx" => {
                rust_common::office_text_extractor::extract_text_from_pptx(file_path.as_str())
            }
            _ => Err("officex不支持的文件格式".into()),
        };
        let r = to_global_result(r);

        r
    })
    .await; //等待完成

    to_global_result(r)?
}

/// @Author: DengLibin
/// @Date: Create in 2024-07-31 13:48:52
/// @Description: 获取文件拓展名
fn get_ext_name(file_path: &str) -> Option<String> {
    if let Some(ext) = Path::new(file_path).extension() {
        if let Some(file_ext) = ext.to_str() {
            return Some(file_ext.to_lowercase());
        }
    }
    None
}
/// @Author: DengLibin
/// @Date: Create in 2024-07-31 14:31:13
/// @Description: 是否是utf8编码
fn is_utf8(buffer: &[u8]) -> bool {
    match std::str::from_utf8(buffer) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// @Author: DengLibin
/// @Date: Create in 2024-10-31 10:00:55
/// @Description: 是否是文本文件
pub async fn is_text_file(file_path: &str) -> GlobalResult<bool> {
    let mut file = to_global_result(File::open(file_path).await)?;

    let mut buffer = [0_u8; 1024]; // 读取前1024字节
    let read_size = to_global_result(file.read(&mut buffer).await)?;

    // 检查是否包含大量非打印字符
    let non_printable_count = buffer[..read_size]
        .iter()
        .filter(|&&byte| byte < 0x09 || (byte > 0x0D && byte < 0x20)) // 非打印字符范围
        .count();

    // 如果非打印字符超过阈值，比如10%，则认为是二进制文件
    Ok(non_printable_count < read_size / 10)
}
