//! @Author: DengLibin
//! @Date: Create in 2024-12-19 13:42:02
//! @Description: 文本处理工具类

use log::error;
use tokio_util::bytes::BufMut;

/// @Author: DengLibin
/// @Date: Create in 2024-12-19 13:42:56
/// @Description: 获取关键词所处的文本片段
pub fn find_text_snippets(
    text: &str,                       //文本
    keywords: &Vec<String>,           //关键词
    snippet_size: usize,              //片段大小（前后各取多少个字符）
    per_key_snippet_max_count: usize, //每个关键词最大取多少个片段
) -> Vec<String> {
    let mut snippets = Vec::new();
    for keyword in keywords {
        // 查找关键词的位置
        let mut start = 0;
        let mut find_count = 0;
        while let Some(pos) = text[start..].find(keyword) {
            // 计算关键词的实际位置
            let actual_pos = start + pos;
            //关键词前面有多少个字符
            let pre_char_count = text[..actual_pos].chars().count();

            //关键词后面有多少个字符
            let next_char_count = text[actual_pos + keyword.len()..].chars().count();
            //跳过前n个字符
            let skip_char_count = if pre_char_count > snippet_size {
                pre_char_count - snippet_size
            } else {
                0
            };
            //关键词后面取snippet_size个字符
            let next_take_char_count = if next_char_count > snippet_size {
                snippet_size
            } else {
                next_char_count
            };

            let take_char_count =
                pre_char_count - skip_char_count + keyword.chars().count() + next_take_char_count;
            let snippet = text
                .chars()
                .skip(skip_char_count)
                .take(take_char_count)
                .collect::<String>();

            snippets.push(snippet);
            find_count += 1;
            if find_count > per_key_snippet_max_count {
                break;
            }

            // 更新start位置，继续查找下一个匹配的关键词
            start = actual_pos + keyword.len();
        }
    }

    snippets
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-19 13:51:06
/// @Description: 用标签把文本中的关键词包裹起来
pub fn wrap_keywords(
    text: &str,             //文本
    keywords: &Vec<String>, //关键词(按顺序包裹)
    open_tag: &str,         //标签
    close_tag: &str,        //关闭标签
) -> String {
    if text.is_empty() {
        return "".into();
    }
    //关键词（开始位置，结束位置）
    let tags = keywords_pos(text, keywords);

    wrap_tags(text, open_tag, close_tag, tags)
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-19 15:52:51
/// @Description: 标签包裹
fn wrap_tags(text: &str, open_tag: &str, close_tag: &str, tags: Vec<(usize, usize)>) -> String {
    let mut content: Vec<u8> = Vec::new();
    let mut i = 0;
    //转字节，因为tag是基于字节序的
    let bytes = text.as_bytes();

    while i < bytes.len() {
        if let Some(tag) = in_which_tag(i, &tags) {
            content.put_slice(open_tag.as_bytes());
            content.put_slice(&bytes[tag.0..tag.1 + 1]);
            content.put_slice(close_tag.as_bytes());
            i = tag.1 + 1;
        } else {
            content.put_slice(&bytes[i..i + 1]);

            i += 1;
        }
    }

    let r = String::from_utf8(content);
    if let Ok(s) = r {
        s
    } else {
        error!("wrap_keywords异常:{}", r.unwrap_err());
        "".into()
    }
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-19 15:54:54
/// @Description: 获取关键词在文本中的位置
pub fn keywords_pos(text: &str, keywords: &Vec<String>) -> Vec<(usize, usize)> {
    let mut tags: Vec<(usize, usize)> = vec![];

    for keyword in keywords {
        //查找关键词位置
        if keyword.is_empty() {
            continue;
        }
        let mut start_index = 0;
        let len = keyword.len();
        while let Some(mut pos) = text[start_index..].find(keyword) {
            //矫正
            pos = start_index + pos;
            //不在已有的标签内
            if !in_tags(pos, pos + len - 1, &tags) {
                tags.push((pos, pos + len - 1));
            }
            start_index = pos + len;
            if start_index >= text.len() {
                break;
            }
        }
    }
    //从小到大排序
    tags.sort_by(|item1, item2| item1.0.cmp(&item2.0));
    tags
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-19 14:07:03
/// @Description: 目标位置是否在已有标签范围中
fn in_tags(pos: usize, pos_end: usize, tags: &Vec<(usize, usize)>) -> bool {
    for ele in tags {
        //开始位置和结束位置只要有一个在已有的标签内就算
        if pos >= ele.0 && pos <= ele.1 {
            return true;
        }
        if pos_end >= ele.0 && pos_end <= ele.1 {
            return true;
        }
    }
    return false;
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-19 14:21:14
/// @Description: 找到目标位置所在标签
fn in_which_tag(pos: usize, tags: &Vec<(usize, usize)>) -> Option<(usize, usize)> {
    for ele in tags {
        if pos >= ele.0 && pos <= ele.0 {
            return Some(*ele);
        }
    }
    return None;
}
