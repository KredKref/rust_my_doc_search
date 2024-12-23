//! @Author: DengLibin
//! @Date: Create in 2024-03-18 14:22:13
//! @Description: rust和c互转

use std::ffi::CStr;

use std::ffi::c_char;

/// @Author: DengLibin
/// @Date: Create in 2024-03-18 14:24:38
/// @Description: c语言 char指针转rust字符串
pub fn c_p_char2_string(c_ptr: *const i8 ) -> Option<String> {
    unsafe {
        let c_str_slice = CStr::from_ptr(c_ptr as *const c_char);
        match c_str_slice.to_str() {
            Ok(string) => {
                return Some(string.into());
            }
            Err(_) => {
                println!("The C string is not valid UTF-8");
                return None;
            }
        };
    }
}
