mod test {
    use extractous::Extractor;
    use rust_my_doc_search::tantivy::{tantivy_jieba, tantivy_search::{self, IndexDocument}};

    #[test]
    pub fn test_oepen_index() {
        tantivy_search::open_index("./indices").unwrap();
    }
    #[test]
    pub fn test_write() {
        let mut index = tantivy_search::open_index("./indices").unwrap();
        let doc = IndexDocument {
            index_dir_id: 1,
            file_path: r"D:\yiscn\20231102四川省环境政策研究与规划院华为云资源汇总.docx"
                .replace("\\", "/"),
            file_name: "20231102四川省环境政策研究与规划院华为云资源汇总.docx".to_string(),
            file_content: "租赁华为云资源情况".to_string(),
        };
        tantivy_search::insert_doc(&mut index, &doc).unwrap();
    }
    #[test]
    pub fn test_search() {
        println!("查询");
        let index = tantivy_search::open_index("./indices").unwrap();
        // let r =tantivy_search::search_doc(&index, "file_name:四川 AND file_content:情况".into(), 1, 10).unwrap();
        let r = tantivy_search::search_doc(
            &index,
            "file_name:大文件成都亿橙".into(),
            1,
            10,
        )
        .unwrap();
        for line in r {
           println!("{:?}", line);
        }
    }
    #[test]
    pub fn test_create_index_doc() {
        let file_path =
            r"D:\yiscn\测试文件\624（新都）技情报知识结构化处理与应用-技术-陪1_4.21(秘密).pdf";
        let extractor = Extractor::new().set_extract_string_max_length(500_000);

        let r = extractor.extract_file_to_string(file_path);
        if let Ok(content) = r {
            let docs: Vec<IndexDocument> = tantivy_search::IndexDocument::split_to_list(file_path.to_string(), content.0, 1);
            // let docs: Vec<IndexDocument> = tantivy_search::IndexDocument::split_to_list(file_path.to_string(), "".into());
            for doc in docs {
                println!("{}", doc.file_content);
                println!("====================================================================");
            }
        }
    }

    #[test]
    pub fn test_tokenize() {
        let tokens = tantivy_jieba::tokenize("数据库");
        for token in tokens.into_iter() {
            println!("{}", token);
        }
    }
}
