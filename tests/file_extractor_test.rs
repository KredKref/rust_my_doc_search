//! @Author: DengLibin
//! @Date: Create in 2024-07-25 16:06:28
//! @Description:
//!
//!
mod test {

    use std::{fs, sync::Arc};

    use log::info;

    use rust_my_doc_search::{
        app::init_log,
        file_scanner::{
            file_extractor::{extract_file, remove_out_dir_all},
            file_text_extractor::{self, FileText}, is_compress_file,
        },
    };
    use tokio::{runtime::Builder, sync::mpsc};
    fn cre_runtime() -> tokio::runtime::Runtime {
        // 创建一个 Tokio 运行时，配置线程池的大小
        Builder::new_multi_thread()
            .worker_threads(1) // 设置工作线程的数量
            .max_blocking_threads(16) // 设置阻塞任务的最大线程数
            .enable_all()
            .build()
            .unwrap()
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-19 08:58:47
    /// @Description: 测试文件提取
    #[test]
    pub fn test_file_extractor() {
        let scan_dir = r#"D:\ce"#;
        let r = cre_runtime();
        let s = rust_common::date::get_sys_timestamp_millis();
        let r = r.block_on(async {
            init_log().await.unwrap();
            info!("日志初始化完成");
            let (tx, mut rx) = mpsc::channel::<String>(1000); // 创建通道，设置缓冲区大小

            //接收文件
            let f1 = async move {
                let mut i = 0;
                //接收提取的文件
                while let Some(file_path) = rx.recv().await {
                    info!("收到文件:{}", file_path);
                    i += 1;
                }
                println!("收到文件完成,数量:{}", i);
            };

            //开始提取文件
            let f2 = extract_file(scan_dir, tx);
            let (_r1, r2) = tokio::join!(f1, f2);
            r2
        });
        let e = rust_common::date::get_sys_timestamp_millis();
        println!("耗时:{} ms", e - s);
        r.unwrap();

        remove_out_dir_all(scan_dir).unwrap();
    }

    //测试文件文本提取
    #[test]
    pub fn test_file_and_text_extractor() {
        let scan_dir = r#"D:\ce"#;
        let r = cre_runtime();
        let s = rust_common::date::get_sys_timestamp_millis();
        let r = r.block_on(async {
            init_log().await.unwrap();
            info!("日志初始化完成");
            let (tx, mut rx) = mpsc::channel::<String>(1000); // 创建通道，设置缓冲区大小

            let (text_sender, mut text_receiver) = mpsc::channel::<FileText>(1); // 创建通道，设置缓冲区大小
            let text_sender_arc = Arc::new(text_sender);
            //接收文件
            let f1 = async move {
                let mut i = 0;
                //接收提取的文件
                while let Some(file_path) = rx.recv().await {
                    info!("收到文件:{}", file_path);
                    i += 1;

                    file_text_extractor::spawn_extract_text(file_path, text_sender_arc.clone())
                        .await;
                }
                println!("收到文件完成,数量:{}", i);
            };

            //接收文件内容
            let f2 = async move {
                info!("开始接收文件内容");
                let mut i = 0;
                while let Some(file_text) = text_receiver.recv().await {
                    log::info!("收到文件内容: {}", file_text.file_path);

                    i += 1;
                    /*
                                        let path =  &file_text.file_path.replace("\\", "/");
                                        let i = path.rfind("/").unwrap_or(0);
                                        let file_name = path[i+1..].to_string();
                                        let r = rust_common::file_util::write_str(

                                            format!("./texts/{}-{}.txt", i, file_name).as_str(),
                                            &file_text.text,
                                        );
                                        if let Err(e) = r {
                                            log::error!("写入文件内容失败: {}", e);
                                        }
                    */
                }
                info!("接收文件内容完成,数量:{}", i);
            };

            //开始提取文件
            let f3 = extract_file(scan_dir, tx);
            let (_r1, _r2, r3) = tokio::join!(f1, f2, f3);
            // let (_r1, r3) = tokio::join!(f1,  f3);
            r3
        });
        let e = rust_common::date::get_sys_timestamp_millis();
        println!("耗时:{} ms", e - s);
        r.unwrap();

        remove_out_dir_all(scan_dir).unwrap();
    }

    #[test]
    pub fn test_remove_out_dir_all() {
        let s = rust_common::date::get_sys_timestamp_millis();
        remove_out_dir_all(r#"D:/ce"#).unwrap();

        let e = rust_common::date::get_sys_timestamp_millis();
        println!("耗时:{} ms", e - s);
    }

    #[test]
    pub fn test_is_compress_file() {
        let b = is_compress_file("c:/test/测试.tar.gz");
        println!("is_compress_file:{}", b)
    }

    #[test]
    pub fn count_file() {
        let count = do_count(r#"D:\yiscn\测试文件"#);
        println!("文件（夹）数量:{}", count);
    }

    fn do_count(file_path: &str) -> usize {
        let metadata = fs::metadata(file_path).unwrap();
        if metadata.is_file() {
            return 1;
        }

        let entrys = fs::read_dir(file_path).unwrap();
        let mut count: usize = 1_usize;
        for entry in entrys {
            let entry = entry.unwrap();
            let path = entry.path();

            let o = path.as_os_str().to_str().unwrap();
            count += do_count(o);
        }
        return count;
    }
}
