//! @Author: DengLibin
//! @Date: Create in 2023-11-06 15:08:30
//! @Description: 全局配置

use std:: fmt::Display;

#[derive(Debug)]
pub struct GlobalError {
    pub msg: String,
}
impl GlobalError {
    pub fn new(msg: String)->Self{
        GlobalError{msg}
    }
}

impl std::fmt::Display for GlobalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

// 自定义结果
pub type GlobalResult<T> = std::result::Result<T, GlobalError>;



//转换结果
pub fn to_global_result<T, E: Display>(result: Result<T, E>) -> GlobalResult<T> {
    return match result {
        Ok(r) => Result::Ok(r),
        Err(err) => {
            // 获取当前调用栈
            //let backtrace = Backtrace::capture();
           // log::error!("异常了:{};\n调用栈:{:?}", err, backtrace);
            Result::Err(GlobalError {
                msg: err.to_string(),
            })
        }
    };
}
