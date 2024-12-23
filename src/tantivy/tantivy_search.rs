//! @Author: DengLibin
//! @Date: Create in 2024/4/14 18:37
//! @Description

use crate::sys::global::{to_global_result, GlobalResult};
use crate::tantivy::tantivy_jieba::JiebaTokenizer;
use egui::TextBuffer;

use regex::Regex;
use rust_common::file_util;
use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;

use tantivy::indexer::LogMergePolicy;
use tantivy::query::QueryParser;
use tantivy::schema::{
    IndexRecordOption, Schema, TextFieldIndexing, TextOptions, Value, INDEXED, STORED, STRING,
};

use tantivy::tokenizer::TextAnalyzer;
use tantivy::{doc, Index, TantivyDocument, Term};

//内容长度
const CONTETN_LEN: usize = 1000;

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexDocument {
    pub index_dir_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_content: String,
}

impl IndexDocument {
    pub fn new() -> Self {
        IndexDocument {
            index_dir_id: 0,
            file_path: "".into(),
            file_name: "".into(),
            file_content: "".into(),
        }
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-11 12:22:47
    /// @Description: 根据内容创建
    pub fn split_to_list(file_path: String, content: String, index_dir_id: i64) -> Vec<Self> {
        let content = content.replace("\n", "").replace("\r", "");

        // 创建一个正则表达式，匹配多个空格
        let re = Regex::new(r"\s+").unwrap();

        // 使用正则替换多个空格为单个空格
        let binding = re.replace_all(&content, " ");
        let content = binding.as_str();

        let contents = Self::split_content(content, CONTETN_LEN);
        let file_path = file_path.replace("\\", "/");
        let mut file_name = String::new();
        let fr = file_path.rfind("/");
        if let Some(i) = fr {
            file_name.push_str(&file_path[i + 1..]);
        }
        contents
            .into_iter()
            .map(|item| Self {
                index_dir_id,
                file_path: file_path.clone(),
                file_name: file_name.clone(),
                file_content: item,
            })
            .collect::<Vec<Self>>()
    }
    /// @Author: DengLibin
    /// @Date: Create in 2024-12-11 12:23:04
    /// @Description: 切割
    fn split_content(content: &str, len: usize) -> Vec<String> {
        let content = content.chars();
        let mut contents: Vec<String> = vec![];

        let mut sub_content = String::new();
        for ch in content {
            if ((ch == '.'
                || ch == ','
                || ch == '。'
                || ch == ','
                || ch == '?'
                || ch == '？'
                || ch == '!'
                || ch == '！')
                && sub_content.len() >= len)
                || sub_content.len() > 2 * len
            {
                sub_content.push(ch);
                contents.push(sub_content);
                sub_content = String::new();
                continue;
            }
            sub_content.push(ch);
        }

        contents.push(sub_content);

        return contents;
    }
}

//字段名
const FIELD_INDEX_DIR_ID: &str = "index_dir_id";
const FIELD_FILE_ANME: &str = "file_name";
const FIELD_FILE_PATH: &str = "file_path";
const FIELD_FILE_CONTENT: &str = "file_content";

//分词器名称
const JIEBA: &str = "jieba";

/// @Author: DengLibin
/// @Date: Create in 2024-04-14 20:08:55
/// @Description: 打开
pub fn open_index(index_dir: &str) -> GlobalResult<Index> {
    let bo = file_util::exist(index_dir);
    let text_analyzer = TextAnalyzer::builder(JiebaTokenizer {}).build();

    let index = if !bo {
        //不存在则创建
        create_index(index_dir).unwrap()
    } else {
        to_global_result(Index::open_in_dir(index_dir))? // 获取索引
    };

    //注册jieba分词器

    index.tokenizers().register(JIEBA, text_analyzer);
    Ok(index)
}

/// @Author: DengLibin
/// @Date: Create in 2024-04-14 20:08:45
/// @Description: 创建索引
fn create_index(index_dir: &str) -> GlobalResult<Index> {
    // 定义Schema
    let bo = file_util::exist(index_dir);
    if !bo {
        to_global_result(file_util::create_dir_all(index_dir))?;
    }
    let mut schema_builder = Schema::builder();

    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer(JIEBA)
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_field_indexing)
        .set_stored();

    schema_builder.add_i64_field(FIELD_INDEX_DIR_ID, INDEXED);
    schema_builder.add_text_field(FIELD_FILE_ANME, text_options.clone());
    schema_builder.add_text_field(FIELD_FILE_CONTENT, text_options);
    schema_builder.add_text_field(FIELD_FILE_PATH, STRING | STORED);

    let schema = schema_builder.build(); // 在目录中创建索引
                                         //let index = Index::create_in_ram(schema.clone()); // 获取索引写入器
    let index = to_global_result(Index::create_in_dir(index_dir, schema.clone()))?; // 获取索引写入器
                                                                                    //let  index_writer = to_global_result(index.writer(50_000_000))?; // 添加文档
                                                                                    // let title = to_global_result(schema.get_field("title"))?;
                                                                                    // let body = to_global_result(schema.get_field("body"))?;
                                                                                    // let doc = doc!(title => "Example Title", body => "This is the body of the document.");
                                                                                    // let _ = index_writer.add_document(doc); // 将文档提交到索引
                                                                                    // let _ = index_writer.commit();

    // let index_reader = to_global_result(index
    //     .reader_builder()
    //     .reload_policy(ReloadPolicy::OnCommitWithDelay)
    //     .try_into())?;

    Ok(index)
}

/// @Author: DengLibin
/// @Date: Create in 2024-04-15 10:34:37
/// @Description: 添加文档
pub fn insert_doc(index_obj: &mut Index, doc: &IndexDocument) -> GlobalResult<()> {
    let index_dir_id = to_global_result(index_obj.schema().get_field(FIELD_INDEX_DIR_ID))?;
    let title = to_global_result(index_obj.schema().get_field(FIELD_FILE_ANME))?;
    let body = to_global_result(index_obj.schema().get_field(FIELD_FILE_CONTENT))?;
    let path = to_global_result(index_obj.schema().get_field(FIELD_FILE_PATH))?;

    let mut tantivy_doc = TantivyDocument::default();

    tantivy_doc.add_i64(index_dir_id, doc.index_dir_id);
    tantivy_doc.add_text(title, &doc.file_name);
    tantivy_doc.add_text(body, &doc.file_content);
    tantivy_doc.add_text(path, &doc.file_path);

    //创建writer
    let mut index_writer: tantivy::IndexWriter = to_global_result(index_obj.writer(50_000_000))?; // 添加文档

    to_global_result(index_writer.add_document(tantivy_doc))?;
    to_global_result(index_writer.commit())?;
    Ok(())
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-10 17:56:29
/// @Description: 批量添加
pub fn insert_doc_list(index_obj: &mut Index, docs: &Vec<IndexDocument>) -> GlobalResult<()> {
    let index_dir_id = to_global_result(index_obj.schema().get_field(FIELD_INDEX_DIR_ID))?;
    let title = to_global_result(index_obj.schema().get_field(FIELD_FILE_ANME))?;
    let body = to_global_result(index_obj.schema().get_field(FIELD_FILE_CONTENT))?;
    let path = to_global_result(index_obj.schema().get_field(FIELD_FILE_PATH))?;

    //创建writer 默认cpu核心数，最大8，会根据指定的缓冲区大小进行矫正
    let mut index_writer = to_global_result(index_obj.writer(200_000_000))?; // 添加文档
    // let mut index_writer = to_global_result(index_obj.writer_with_num_threads(16, 320_000_000))?; // 添加文档
    
    let merge_policy = LogMergePolicy::default();
    index_writer.set_merge_policy(Box::new(merge_policy));

    let mut i = 0;

    for doc in docs {
        i += 1;
        let mut tantivy_doc = TantivyDocument::default();
        tantivy_doc.add_i64(index_dir_id, doc.index_dir_id);
        tantivy_doc.add_text(title, &doc.file_name);
        tantivy_doc.add_text(body, &doc.file_content);
        tantivy_doc.add_text(path, &doc.file_path);
        to_global_result(index_writer.add_document(tantivy_doc))?;
        if i % 10000 == 0 {
            to_global_result(index_writer.commit())?;
         
        }
    }
    to_global_result(index_writer.commit())?;
    Ok(())
}
/// @Author: DengLibin
/// @Date: Create in 2024-04-15 10:34:44
/// @Description: 查询
pub fn search_doc(
    index_obj: &Index,
    query_str: &str,
    page_num: usize,
    page_size: usize,
) -> GlobalResult<Vec<IndexDocument>> {
    let schema = index_obj.schema();
    let title = to_global_result(schema.get_field(FIELD_FILE_ANME))?;
    let body = to_global_result(schema.get_field(FIELD_FILE_CONTENT))?;

    let index_reader = to_global_result(index_obj.reader())?;

    let searcher = index_reader.searcher();
    let query_parser = QueryParser::for_index(index_obj, vec![title, body]);

    let query = to_global_result(query_parser.parse_query(query_str.as_str()))?;
    let top_doc = TopDocs::with_limit(page_size).and_offset((page_num - 1) * page_size);
    // let top_doc = TopDocs::with_limit(10);
    // 获取命中总数
    // let hits_total = to_global_result(searcher.search(&query, &tantivy::collector::Count))?;
    let top_docs = to_global_result(searcher.search(&query, &top_doc))?;

    let vec: Vec<IndexDocument> = to_index_docs(top_docs, searcher, schema)?;

    Ok(vec)
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-13 11:02:38
/// @Description: 获取结果
fn to_index_docs(
    top_docs: Vec<(f32, tantivy::DocAddress)>,
    searcher: tantivy::Searcher,
    schema: Schema,
) -> Result<Vec<IndexDocument>, crate::sys::global::GlobalError> {
    let mut vec = Vec::new();
    for (_score, doc_address) in top_docs {
        let mut index_doc = IndexDocument::new();

        // let id = doc_address.doc_id.to_string();

        let tantivy_document: TantivyDocument = to_global_result(searcher.doc(doc_address))?;
        let field_values = tantivy_document.field_values();
        for field_value in field_values {
            let field_name = schema.get_field_name(field_value.field());

            match field_name {
                FIELD_INDEX_DIR_ID => {
                    index_doc.index_dir_id = field_value.value().as_i64().unwrap();
                }
                FIELD_FILE_ANME => {
                    index_doc
                        .file_name
                        .push_str(field_value.value().as_str().unwrap_or(""));
                }
                FIELD_FILE_PATH => {
                    index_doc
                        .file_path
                        .push_str(field_value.value().as_str().unwrap_or(""));
                }
                FIELD_FILE_CONTENT => {
                    index_doc
                        .file_content
                        .push_str(field_value.value().as_str().unwrap_or(""));
                }

                _ => {}
            }
        }
        vec.push(index_doc);
    }
    Ok(vec)
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-12 14:15:54
/// @Description: 移除
pub fn delete_by_index_dir_id(index_obj: &mut Index, index_dir_id: i64) -> GlobalResult<()> {
    let mut index_writer: tantivy::IndexWriter = to_global_result(index_obj.writer(50_000_000))?;
    let index_dir_id_field = to_global_result(index_obj.schema().get_field(FIELD_INDEX_DIR_ID))?;
    // 删除条件：id = "1"
    let term = Term::from_field_i64(index_dir_id_field, index_dir_id);
    index_writer.delete_term(term); // 标记删除

    // 提交删除操作
    let _r = to_global_result(index_writer.commit())?; // 必须提交以使删除生效
    Ok(())
}
