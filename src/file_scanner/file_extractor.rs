//! @Author: DengLibin
//! @Date: Create in 2024-07-25 10:44:30
//! @Description: 文件提取器，输入： 文件路径（文件，文件夹，压缩包），输出：文件列表（文档文件）,使用通道接收
//!

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::component::comp_7z::decompress_file;
use crate::sys::global::{to_global_result, GlobalResult};

use log::{error, info};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Semaphore};
use tokio::{fs, task};

use super::is_compress_file;

/// @Author: DengLibin
/// @Date: Create in 2024-07-26 10:15:53
/// @Description: 提取文件
/// @params: seven_z_path 7z 压缩解压工具路径
/// @params: file_path 文件路径
/// @params: concurrency_num
/// @params: file_sender 通道发送者，提取到一个文件会通过该发送者发送出去
/// @return: GlobalResult<()>
/// 调用示例:
/// ```
/// let r = cre_runtime();
/// let s = rust_common::date::get_sys_timestamp_millis();
/// let r = r.block_on(async {
///     let (tx, mut rx) = mpsc::channel::<String>(1000); // 创建通道，设置缓冲区大小
///     let f1 = async move {
///         //接收提取的文件
///         while let Some(file_path) = rx.recv().await {
///             info!("收到文件:{}", file_path);
///             // rust_common::tokio_file::delete_file_or_dir(&file_path).await;
///         }
///         println!("收到文件完成")
///     };
///     let f2 = extract_file(r#"D:\software\7z\7z.exe"#, r#"D:\yiscn\测试文件"#, tx);
///     let (_r1, r2) = tokio::join!(f1, f2);
///     r2
/// });
/// let e = rust_common::date::get_sys_timestamp_millis();
/// println!("耗时:{} ms", e - s);
/// r.unwrap();
/// ```
/// @return: GlobalResult<()>
pub async fn extract_file(file_path: &str, file_sender: Sender<String>) -> GlobalResult<()> {
    let r = remove_out_dir_all(file_path);
    if let Err(e) = r {
        error!("删除out解压目录异常:{}", e);
    }
    let extractor = FileExtractor::new(r#"./7z/7z.exe"#, 10000);
    extractor.start(file_path.into(), file_sender).await
}

struct FileExtractor {
    seven_z_path: String,
    semaphore: Arc<Semaphore>,  //信号量 控制并发数
    tx: mpsc::Sender<String>,   // 通道发送者
    rx: mpsc::Receiver<String>, // 通道接收者
}

impl FileExtractor {
    fn new(seven_z_path: &str, concurrency_num: usize) -> Self {
        let semaphore = Arc::new(Semaphore::new(concurrency_num));
        let (tx, rx) = mpsc::channel::<String>(100); // 创建通道，设置缓冲区大小

        FileExtractor {
            seven_z_path: seven_z_path.into(),
            semaphore,
            tx,
            rx,
        }
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-07-25 15:40:05
    /// @Description: 抽取一个文件(文件夹，压缩包, 普通文件)
    /// @params: file_path 文件路径
    /// @params: file_sender 文件发送者
    pub async fn start(self, file_path: String, file_sender: Sender<String>) -> GlobalResult<()> {
        let Self {
            seven_z_path,
            semaphore,
            tx,
            mut rx,
        } = self;

        //计数器
        let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0_usize));

        //复制发送者
        let tx = Arc::new(tx);
        let seven_path_a = Arc::new(seven_z_path);
        let file_sender = Arc::new(file_sender);
        //向通道发送一个文件路径
        send_path(file_path, tx.clone(), counter.clone()).await?;
        let mut file_count = 0_usize;

        //从通道获取文件路径，通道未关闭：有数据读取成功，无数据 阻塞， 通道关闭：有数据读取成功，无数据返回None
        while let Some(file_path) = rx.recv().await {
            if file_path.is_empty() {
                //空的表示完了
                break;
            }
            file_count += 1;
            //获取执行权
            let permit = semaphore.clone().acquire_owned().await;

            if let Ok(p) = permit {
                let tx = tx.clone();
                let seven_path_a = seven_path_a.clone();
                let counter = counter.clone();
                let file_sender = file_sender.clone();
                tokio::task::spawn(async move {
                    // log::info!("do_extract_file:::::{}", file_path);
                    let r =
                        do_extract_file(seven_path_a, file_path.clone(), tx, counter, file_sender)
                            .await;
                    if r.is_err() {
                        log::error!("文件提取失败:{},{}", r.err().unwrap(), file_path);
                    }
                    // tokio::time::sleep(Duration::from_secs(2)).await;
                    drop(p);
                });
            } else {
                log::error!("获取执行权失败:{}", file_path);
            }
        }
        info!("扫描文件完成:文件(夹)数量:{}", file_count);
        Ok(())
    }
}
/// @Author: DengLibin
/// @Date: Create in 2024-07-25 11:13:54
/// @Description: 执行提取
async fn do_extract_file(
    seven_z_path: Arc<String>,
    file_path: String,
    tx: Arc<Sender<String>>,
    counter: Arc<AtomicUsize>,
    file_sender: Arc<Sender<String>>,
) -> GlobalResult<()> {
    let metadata_r = fs::metadata(&file_path).await;
    let meta = to_global_result(metadata_r)?;
    //文件夹
    if meta.is_dir() {
        //文件夹本身发送出去
        if !file_path.ends_with(".out") {
            to_global_result(file_sender.send(file_path.clone()).await)?;
        }
        //提取子文件(夹)
        let mut entries = to_global_result(fs::read_dir(file_path).await)?;
        //遍历子文件（夹）发送出去
        while let Some(entry) = to_global_result(entries.next_entry().await)? {
            let path = entry.path();
            if let Some(p) = path.to_str() {
                //发送出去
                send_path(p.into(), tx.clone(), counter.clone()).await?;
            }
        }
        //计数器减1（表示当前文件路径已消费）,在发送完成后再减，减到0表示递归完成
        if decrement_counter(&counter) {
            //完成，发送一个""
            send_path("".into(), tx.clone(), counter.clone()).await?;
        }

        return Ok(());
    }

    //文件
    if meta.is_file() {
        //1.压缩包: 解压到文件夹，提取该文件夹
        if is_compress_file(file_path.as_str()) {
            //压缩文件本身发送出去
            to_global_result(file_sender.send(file_path.clone()).await)?;
            return decompress_and_send(
                seven_z_path,
                file_path.clone(),
                tx.clone(),
                counter.clone(),
            )
            .await;
        }
        //2.普通文件: 提取文件
        //提取到的文件发送出去
        to_global_result(file_sender.send(file_path).await)?;
        //计数器减1
        if decrement_counter(&counter) {
            //完成，发送一个""
            send_path("".into(), tx.clone(), counter.clone()).await?;
        }

        return Ok(());
    }

    //符号链接
    if meta.is_symlink() {
        //获取实际的文件路径
        let target_path = to_global_result(fs::read_link(&file_path).await)?;
        let path = target_path.to_str();
        if let Some(real_path) = path {
            //发送出去
            send_path(real_path.into(), tx.clone(), counter.clone()).await?;
        }
        //计数器减1
        if decrement_counter(&counter) {
            //完成，发送一个""
            send_path("".into(), tx.clone(), counter.clone()).await?;
        }
        return Ok(());
    }

    Ok(())
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-19 08:48:09
/// @Description: 解压缩文件，并发送出去
async fn decompress_and_send(
    seven_z_path: Arc<String>,
    file_path: String,
    tx: Arc<Sender<String>>,
    counter: Arc<AtomicUsize>,
) -> GlobalResult<()> {
    //解压路径
    let out_dir = format!("{}.out", file_path);

    let tx = tx.clone();
    let tx2 = tx.clone();

    let counter2 = counter.clone();
    //解压 耗时任务 交给阻塞线程池
    task::spawn_blocking(move || {
        let r = decompress_file(seven_z_path.as_str(), file_path.as_str(), out_dir.as_str());
        match r {
            //解压成功
            Ok(()) => {
                //异步发送
                task::spawn(async move {
                    let r = send_path(out_dir.clone(), tx, counter).await;
                    if let Err(e) = r {
                        error!("发送路径失败:{}, {}", out_dir, e);
                    }

                    //计数器减1（表示当前文件路径已消费）
                    if decrement_counter(&counter2) {
                        tokio::spawn(async {
                            //完成，发送一个""
                            let r = send_path("".into(), tx2, counter2).await;
                            if let Err(e) = r {
                                error!("发送空文件路径异常:{}", e);
                            }
                        });
                    }
                });
            }

            //解压异常
            Err(e) => {
                error!("解压异常:{},{}", file_path, e);

                //计数器减1（表示当前文件路径已消费）
                if decrement_counter(&counter2) {
                    tokio::spawn(async {
                        //完成，发送一个""
                        let r = send_path("".into(), tx2, counter2).await;
                        if let Err(e) = r {
                            error!("发送空文件路径异常:{}", e);
                        }
                    });
                }
            }
        }
    });
    Ok(())
}

//发送文件路径，计数器+1
async fn send_path(
    file_path: String,
    tx: Arc<Sender<String>>,
    counter: Arc<AtomicUsize>,
) -> GlobalResult<()> {
    //发送出去
    let r = tx.send(file_path).await;
    to_global_result(r)?;
    increment_counter(&counter); //计数器加1

    Ok(())
}
// 增加计数器
fn increment_counter(counter: &Arc<AtomicUsize>) {
    counter.fetch_add(1, Ordering::SeqCst);
}

// 减少计数器,返回true表示计数器为0
fn decrement_counter(counter: &Arc<AtomicUsize>) -> bool {
    //返回操作前的值
    return counter.fetch_sub(1, Ordering::SeqCst) == 1;
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-02 16:22:15
/// @Description: 删除生成的临时文件夹 .out
pub fn remove_out_dir_all<P: AsRef<Path>>(path: P) -> GlobalResult<()> {
    let p = path.as_ref().to_str();
    if let Some(dir) = p {
        if dir.ends_with(".out") {
            return remove_dir_all(dir);
        }
    }

    if path.as_ref().is_dir() {
        for entry in to_global_result(std::fs::read_dir(&path))? {
            let entry = to_global_result(entry)?;
            let path = entry.path();
            if path.is_dir() {
                remove_out_dir_all(path)?;
            }
        }
    }

    Ok(())
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-02 16:41:13
/// @Description: 删除文件夹
fn remove_dir_all<P: AsRef<Path>>(path: P) -> GlobalResult<()> {
    // 检查当前目录是否是软链接，跳过软链接目录
    if path.as_ref().is_symlink() {
        return Ok(()); // 跳过软链接，不进行递归删除
    }

    to_global_result(rust_common::file_util::remove_dir_all(path))
}
