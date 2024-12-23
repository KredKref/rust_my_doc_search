use std::path::Path;

pub mod file_extractor;
pub mod file_text_extractor;

 /// @Author: DengLibin
 /// @Date: Create in 2024-12-19 12:05:04
 /// @Description: 是否压缩文件
pub fn is_compress_file(file_path: &str) -> bool {
    if let Some(ext) = Path::new(file_path).extension() {
        if let Some(file_ext) = ext.to_str() {
            let file_ext = file_ext.to_lowercase();
            return file_ext == "zip"
                || file_ext == "rar"
                || file_ext == "7z"
                || file_ext == "tar"
                || file_ext == "wim"
                || file_ext == "xz"
                || file_ext == "cab"
                || file_ext == "iso"
                || file_ext == "msi"
                || file_ext == "rpm"
                || file_ext == "gz";
        }
    }
    false
}