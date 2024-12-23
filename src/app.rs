//! @Author: DengLibin
//! @Date: Create in 2024-07-17 13:41:12
//! @Description:

use std::{path::PathBuf, sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
}};

use log::{error, info};
use open::that;
use rust_common::{file_util, log_util};

use tokio::task;

use crate::sys::global::{to_global_result, GlobalError, GlobalResult};

///全局状态
pub struct AppState {
    //异步阻塞任务数量
    pub async_task_num: AtomicUsize,
}

/// @Author: DengLibin
/// @Date: Create in 2023-11-06 15:10:01
/// @Description: 日志初始化
pub async fn init_log() -> GlobalResult<()> {
    let r = log_util::init_log(r"log4rs.yaml");
    to_global_result(r)
}



/// @Author: DengLibin
/// @Date: Create in 2023-11-06 15:10:09
/// @Description: 初始化
pub async fn init() -> GlobalResult<AppState> {
    init_log().await?;
    info!("日志初始化完成");
    Ok(AppState {
        async_task_num: AtomicUsize::new(0),
    })
}

/// @Author: DengLibin
/// @Date: Create in 2024-07-18 11:01:34
/// @Description: 后台执行阻塞(耗时)任务,在阻塞线程池中执行,多余任务排队,不会占用工作线程
pub async fn execute_blocking_task<F>(task: F, task_name: String, app_state: Arc<AppState>)
where
    F: FnOnce() + Send + 'static,
{
    app_state.async_task_num.fetch_add(1, Ordering::SeqCst);
    task::spawn_blocking(move || {
        info!(
            "执行异步阻塞任务:{},任务数量:{}",
            task_name,
            app_state.async_task_num.load(Ordering::SeqCst)
        );
        task();
        app_state.async_task_num.fetch_sub(1, Ordering::SeqCst);
        info!(
            "执行异步阻塞任务:{}完成,剩余任务数量:{}",
            task_name,
            app_state.async_task_num.load(Ordering::SeqCst)
        );
    });
}
/// @Author: DengLibin
/// @Date: Create in 2024-07-18 11:02:00
/// @Description: 后台执行非阻塞任务,工作线程中执行,不要执行阻塞(耗时)任务,如果提交了大量阻塞或耗时任务会导致工作线程被占满,无法执行其他任务,导致其他任务等待
pub async fn execute_task<F>(task: F, task_name: String, app_state: Arc<AppState>)
where
    F: FnOnce() + Send + 'static,
{
    app_state.async_task_num.fetch_add(1, Ordering::SeqCst);
    task::spawn(async move {
        info!(
            "执行异步任务:{},任务数量:{}",
            task_name,
            app_state.async_task_num.load(Ordering::SeqCst)
        );
        task();
        app_state.async_task_num.fetch_sub(1, Ordering::SeqCst);
        info!(
            "执行异步任务:{}完成,剩余任务数量:{}",
            task_name,
            app_state.async_task_num.load(Ordering::SeqCst)
        );
    });
}

 /// @Author: DengLibin
 /// @Date: Create in 2024-12-16 11:42:28
 /// @Description: 获取用户主目录
pub fn get_user_home()->GlobalResult<String> {
    let home = dirs_next::home_dir();
    if let Some(path) = home{
        return Ok(path.as_os_str().to_str().unwrap().into());
    }
    return Err(GlobalError::new("获取用户主目录失败".into()));
}
 /// @Author: DengLibin
 /// @Date: Create in 2024-12-16 11:52:17
 /// @Description: 获取数据目录
pub fn get_data_dir()->String {
    let home_dir = get_user_home().unwrap_or("".into());
    let data_dir = format!("{}/.my_search", home_dir);
    let bo = file_util::exist(&data_dir);
    if !bo {
        let r  = file_util::create_dr(&data_dir);
        if let Err(e) = r {
            error!("创建数据目录异常:{}", e);
            return  "".into();
        }
    }
    return data_dir;

}
 /// @Author: DengLibin
 /// @Date: Create in 2024-12-17 10:10:21
 /// @Description: 打开文件所在文件夹
pub fn open_file_folder(file_path: &str) {
    let binding = PathBuf::from(file_path);
    let file_dir = binding.parent();
    if let Some(dir) = file_dir {
        let _ = that(dir);
    }
}