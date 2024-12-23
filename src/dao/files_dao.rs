use egui::ahash::HashSet;

use log::error;
/// @Author: DengLibin
/// @Date: Create in 2024-12-06 13:53:13
/// @Description: 文件
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Row, Sqlite, SqlitePool};

use crate::sys::global::{to_global_result, GlobalError, GlobalResult};

//索引的文件夹
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct IndexFile {
    pub id: i64,
    pub name: String,      //名称
    pub path: String,      //路径
    pub scan_time: i64,    //扫描时间
    pub status: i32,       //状态
    pub index_dir_id: i64, // 文件夹id
}
impl IndexFile {
    /// @Author: DengLibin
    /// @Date: Create in 2024-12-09 10:31:29
    /// @Description: 新建
    /// @param file_path: 文件路径
    /// @param index_idr_id: 索引文件夹id
    pub fn new(file_path: String, index_dir_id: i64) -> Self {
        let file_path = file_path.replace("\\", "/");
        let mut file_name = String::new();
        let fr = file_path.rfind("/");
        if let Some(i) = fr {
            file_name.push_str(&file_path[i + 1..]);
        }
        let time = rust_common::date::get_sys_timestamp_millis();
        Self {
            id: 0,
            name: file_name,
            path: file_path,
            scan_time: time as i64,
            status: 0,
            index_dir_id,
        }
    }
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-03 16:24:49
/// @Description: 建表
pub async fn create_index_file_table(pool: &SqlitePool) -> GlobalResult<()> {
    let mut tx = to_global_result(pool.begin().await)?;
    let sql = r#"CREATE TABLE IF NOT EXISTS INDEX_FILE(id integer  PRIMARY KEY AUTOINCREMENT, 
        "name" text,
        "path" text,
        "scan_time" integer,
        "status" integer,
        "index_dir_id" integer
        )"#;

    let query = sqlx::query::<Sqlite>(sql);

    let r = query.execute(&mut *tx).await;
    if let Err(err) = r {
        // 回滚事务
        to_global_result(tx.rollback().await)?;
        return Err(GlobalError {
            msg: err.to_string(),
        });
    } else {
        //提交
        to_global_result(tx.commit().await)?;
    }
    Ok(())
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-03 17:01:14
/// @Description: 查询所有
pub async fn select_all(pool: &SqlitePool) -> GlobalResult<Vec<IndexFile>> {
    let sql = r#"SELECT * FROM INDEX_FILE ORDER BY id ASC"#;

    let query = sqlx::query_as::<Sqlite, IndexFile>(sql);
    let all = to_global_result(query.fetch_all(pool).await)?;

    Ok(all)
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-09 10:06:07
/// @Description: 查询
pub async fn select_by_index_dir_id(
    pool: &SqlitePool,
    index_dir_id: i64, //所属索引文件夹id
) -> GlobalResult<Vec<IndexFile>> {
    let sql = r#"SELECT * FROM INDEX_FILE  where index_dir_id=$1 ORDER BY  id ASC limit 100"#;

    let query = sqlx::query_as::<Sqlite, IndexFile>(sql).bind(index_dir_id);
    let all = to_global_result(query.fetch_all(pool).await)?;

    Ok(all)
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-12 14:55:14
/// @Description:
pub async fn file_paths(
    pool: &SqlitePool,
    index_dir_id: i64, //所属索引文件夹id
) -> GlobalResult<HashSet<String>> {
    let sql = r#"SELECT path FROM INDEX_FILE  where index_dir_id=$1"#;

    let rows = to_global_result(
        sqlx::query::<Sqlite>(sql)
            .bind(index_dir_id)
            .fetch_all(pool)
            .await,
    )?;

    let paths: HashSet<String> = rows
        .iter()
        .map(|row: &sqlx::sqlite::SqliteRow| row.get::<String, _>("path"))
        .collect();
    Ok(paths)
}
 /// @Author: DengLibin
 /// @Date: Create in 2024-12-16 12:26:44
 /// @Description: 总数
pub async fn count(
    pool: &SqlitePool,
) -> i64{
    let sql = r#"SELECT count(1) FROM INDEX_FILE"#;

    let count = to_global_result(
        sqlx::query::<Sqlite>(sql)
            .map(|row| row.get::<i64, _>(0)) // 从结果中提取第 0 列的值
            .fetch_one(pool)
            .await,
    );
    if let Ok(c) = count {
        c
    }else {
        error!("统计总数异常:{}", count.unwrap_err());
        0
    }
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-03 17:14:50
/// @Description: 添加
pub async fn add_index_file(
    pool: &SqlitePool,
    file_path: String,
    index_dir_id: i64,
) -> GlobalResult<()> {
    let file_path = file_path.replace("\\", "/");
    let mut file_name = String::new();
    let fr = file_path.rfind(".");
    if let Some(i) = fr {
        file_name.push_str(&file_path[i + 1..]);
    }
    let time = rust_common::date::get_sys_timestamp_millis();

    let mut tx = to_global_result(pool.begin().await)?;
    let sql = r#"INSERT INTO INDEX_FILE("name", "path", "scan_time", "status", "index_dir_id")
        VALUES($1, $2, $3, $4, $5)"#;

    let mut query = sqlx::query::<Sqlite>(sql);
    query = query
        .bind(file_name)
        .bind(file_path)
        .bind(time as i64)
        .bind(0)
        .bind(index_dir_id);

    let r = query.execute(&mut *tx).await;
    if let Err(err) = r {
        // 回滚事务
        to_global_result(tx.rollback().await)?;
        return Err(GlobalError {
            msg: err.to_string(),
        });
    } else {
        //提交
        to_global_result(tx.commit().await)?;
    }
    Ok(())
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-09 10:46:24
/// @Description: 批量插入
pub async fn insert_batch(pool: &SqlitePool, index_files: Vec<IndexFile>) -> GlobalResult<()> {
    let chunk_size = 1000; // 每批次插入 1000 条
    let sql = r#"INSERT INTO INDEX_FILE("name", "path", "scan_time", "status", "index_dir_id")"#;
    for chunk in index_files.chunks(chunk_size) {
        let mut tx = to_global_result(pool.begin().await)?;
        let mut query_builder = QueryBuilder::new(sql);

        query_builder.push_values(chunk, |mut builder, index_file| {
            let IndexFile {
                id: _,
                name,
                path,
                scan_time,
                status,
                index_dir_id,
            } = index_file;

            builder
                .push_bind(name)
                .push_bind(path)
                .push_bind(scan_time)
                .push_bind(status)
                .push_bind(index_dir_id);
        });

        to_global_result(query_builder.build().execute(&mut *tx).await)?;
        to_global_result(tx.commit().await)?;
    }
    Ok(())
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-03 17:14:50
/// @Description: 删除
pub async fn delete(pool: &SqlitePool, id: i64) -> GlobalResult<()> {
    let mut tx = to_global_result(pool.begin().await)?;
    let sql = r#"DELETE FROM INDEX_FILE WHERE id=?"#;

    let mut query = sqlx::query::<Sqlite>(sql);
    query = query.bind(id);

    let r = query.execute(&mut *tx).await;
    if let Err(err) = r {
        // 回滚事务
        to_global_result(tx.rollback().await)?;
        return Err(GlobalError {
            msg: err.to_string(),
        });
    } else {
        //提交
        to_global_result(tx.commit().await)?;
    }
    Ok(())
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-09 17:05:58
/// @Description: 删除
pub async fn delete_by_index_dir(pool: &SqlitePool, index_dir_id: i64) -> GlobalResult<()> {
    let mut tx = to_global_result(pool.begin().await)?;
    let sql = r#"DELETE FROM INDEX_FILE WHERE index_dir_id=?"#;

    let mut query = sqlx::query::<Sqlite>(sql);
    query = query.bind(index_dir_id);

    let r = query.execute(&mut *tx).await;
    if let Err(err) = r {
        // 回滚事务
        to_global_result(tx.rollback().await)?;
        return Err(GlobalError {
            msg: err.to_string(),
        });
    } else {
        //提交
        to_global_result(tx.commit().await)?;
    }
    Ok(())
}
