//! @Author: DengLibin
//! @Date: Create in 2024-12-03 16:28:08
//! @Description:
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Sqlite, SqlitePool};

use crate::sys::global::{to_global_result, GlobalError, GlobalResult};

//索引的文件夹
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct IndexDir {
    pub id: i64,
    pub path: String,
}

/// @Author: DengLibin
/// @Date: Create in 2024-12-03 16:24:49
/// @Description: 建表 index_dri
pub async fn create_index_dir_table(pool: &SqlitePool) -> GlobalResult<()> {
    let mut tx = to_global_result(pool.begin().await)?;
    let sql =
        r#"CREATE TABLE IF NOT EXISTS INDEX_DIR(id integer  PRIMARY KEY AUTOINCREMENT, path text)"#;

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
pub async fn select_all(pool: &SqlitePool) -> GlobalResult<Vec<IndexDir>> {
    let sql = r#"SELECT * FROM INDEX_DIR ORDER BY ID ASC"#;

    let query = sqlx::query_as::<Sqlite, IndexDir>(sql);
    let all = to_global_result(query.fetch_all(pool).await)?;

    Ok(all)
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-03 17:14:50
/// @Description: 添加
pub async fn add_index_dir(pool: &SqlitePool, dir_path: String) -> GlobalResult<i64> {
    let mut tx = to_global_result(pool.begin().await)?;
    let sql = r#"INSERT INTO INDEX_DIR(path)VALUES($1)"#;

    let mut query = sqlx::query::<Sqlite>(sql);
    query = query.bind(dir_path);

    let r = query.execute(&mut *tx).await;
    if let Err(err) = r {
        // 回滚事务
        to_global_result(tx.rollback().await)?;
        return Err(GlobalError {
            msg: err.to_string(),
        });
    }
    if let Ok(qr) = r {
        let id = qr.last_insert_rowid();
        //提交
        to_global_result(tx.commit().await)?;
        return Ok(id);
    }
    Ok(0)
}
/// @Author: DengLibin
/// @Date: Create in 2024-12-03 17:14:50
/// @Description: 删除
pub async fn delete(pool: &SqlitePool, id: i64) -> GlobalResult<()> {
    let mut tx = to_global_result(pool.begin().await)?;
    let sql = r#"DELETE FROM INDEX_DIR WHERE id=?"#;

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
