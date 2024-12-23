 //! @Author: DengLibin
 //! @Date: Create in 2024-12-19 14:26:34
 //! @Description: 

 mod test {
    use rust_my_doc_search::util::text_utils::wrap_keywords;

    #[test]
    pub fn test_wrap_keywords(){
        let text = "数据库是一种数据存储系统";
      
        let keywords = vec!["数据库".to_string(), "".to_string()];
        let new_str = wrap_keywords(text, &keywords, "<tag>", "</tag>");

        println!("{}",  new_str);
    }
 }