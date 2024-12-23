 //! @Author: DengLibin
 //! @Date: Create in 2024-12-17 09:30:13
 //! @Description: 

mod test {
    use rust_my_doc_search::component::cmd::open_folder_and_select_file;


    #[test]
    pub fn test_open_forder(){
        open_folder_and_select_file(r"D:/yiscn/测试文件/ppt/ppt/四川电信5G赋能交通行业指引v1.0.pptx".replace("/", "\\").as_str());
    }
}