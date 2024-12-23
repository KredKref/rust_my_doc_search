//! @Author: DengLibin
//! @Date: Create in 2024-11-20 17:09:51
//! @Description:

//不要控制台
#![windows_subsystem = "windows"]

use rust_my_doc_search::{
    sys::global::{to_global_result, GlobalResult},
    ui::main_win::show_win,
};
use tokio::runtime::Builder;
/// @Author: DengLibin
/// @Date: Create in 2024-11-20 17:42:19
/// @Description: 创建tokio运行时
fn cre_tokio_runtime(max_blocking_treads: usize) -> tokio::runtime::Runtime {
    // 创建一个 Tokio 运行时，配置线程池的大小
    Builder::new_multi_thread()
        //.worker_threads(1) // 设置工作线程的数量
        .max_blocking_threads(max_blocking_treads) // 设置阻塞任务的最大线程数
        .enable_all()
        .build()
        .unwrap()
}

fn start_ui() -> GlobalResult<()> {
    // 创建一个 Tokio 运行时，配置线程池的大小
    let runtime = cre_tokio_runtime(16);
    let r = show_win(runtime);
    to_global_result(r)
}

fn main() -> GlobalResult<()> {
    // async_run_web()

    start_ui()
}
