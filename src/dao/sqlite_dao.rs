//! @Author: DengLibin
//! @Date: Create in 2024-11-05 10:02:05
//! @Description: sqlite

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, FromRow, Sqlite, SqlitePool};

use crate::sys::global::{to_global_result, GlobalError, GlobalResult};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SqliteUser {
    pub id: i64,
    pub name: Option<String>,
}



/// @Author: DengLibin
/// @Date: Create in 2024-11-05 10:15:28
/// @Description: 获取sqlite连接池
/// @params: url 链接地址: sqlite:test.db
pub async fn get_conn_pool(url: &str) -> GlobalResult<SqlitePool> {
    let options: SqliteConnectOptions = to_global_result(url.parse())?;
    let options = options.create_if_missing(true);

    let r: Result<SqlitePool, sqlx::Error> = SqlitePool::connect_with(options).await;
    to_global_result(r)
}



/// @Author: DengLibin
/// @Date: Create in 2024-11-05 10:36:26
/// @Description: 创建表
pub async fn create_tabale_demo(pool: &SqlitePool) -> GlobalResult<()> {
    let mut tx = to_global_result(pool.begin().await)?;
    let sql = r#"CREATE TABLE USER(id integer, name text)"#;

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
/// @Date: Create in 2024-11-05 10:41:12
/// @Description: 添加
pub async fn add_demo(pool: &SqlitePool) -> GlobalResult<()> {
    let mut tx = to_global_result(pool.begin().await)?;
    let sql = r#"INSERT INTO USER(id , name)VALUES($1, $2)"#;

    let mut query = sqlx::query::<Sqlite>(sql);
    query = query.bind(1).bind("张三");

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
 /// @Date: Create in 2024-11-05 11:43:38
 /// @Description: 查询
pub async fn select_demo(pool: &SqlitePool) -> GlobalResult<()> {
    let sql = r#"SELECT * FROM USER"#;

    let query = sqlx::query_as::<Sqlite, SqliteUser>(sql);
    let all = to_global_result(query.fetch_all(pool).await)?;
    for row in all {
        println!("{:?}", row);
    }
    Ok(())
}
