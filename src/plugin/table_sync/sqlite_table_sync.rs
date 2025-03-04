use rbs::Value;
use crate::Error;
use rbdc::db::{Connection, ExecResult};
use rbs::value::map::ValueMap;
use crate::executor::{Executor, RBatisConnExecutor};
use crate::table_sync::TableSync;

pub struct SqliteTableSync {}

fn type_str(v: &Value) -> &'static str {
    match v {
        Value::Null => "NULL",
        Value::Bool(_) => "BOOLEAN",
        Value::I32(_) => "INTEGER",
        Value::I64(_) => "INT8",
        Value::U32(_) => "INTEGER",
        Value::U64(_) => "INT8",
        Value::F32(_) => "DOUBLE",
        Value::F64(_) => "DOUBLE",
        Value::String(_) => "TEXT",
        Value::Binary(_) => "BLOB",
        Value::Array(_) => "NULL",
        Value::Map(_) => "NULL",
        Value::Ext(t, v) => match *t {
            "Date" => "TEXT",
            "DateTime" => "TEXT",
            "Time" => "TEXT",
            "Timestamp" => "INT8",
            "Decimal" => "NUMERIC",
            "Json" => "BLOB",
            "Uuid" => "TEXT",
            _ => "NULL",
        },
    }
}

#[async_trait::async_trait]
impl TableSync for SqliteTableSync {
    async fn sync(&self, mut rb: RBatisConnExecutor, table: Value, name: &str) -> Result<(), Error> {
        match table {
            Value::Map(m) => {
                let mut sql_create = format!("CREATE TABLE {} ", name);
                let mut sql_column = format!("");
                for (k, v) in &m {
                    let k = k.as_str().unwrap_or_default();
                    sql_column.push_str(k);
                    sql_column.push_str(" ");
                    sql_column.push_str(type_str(&v));
                    if k.eq("id") || v.as_str().unwrap_or_default() == "id" {
                        sql_column.push_str(" PRIMARY KEY NOT NULL ");
                    }
                    sql_column.push_str(",");
                }
                if sql_column.ends_with(",") {
                    sql_column = sql_column.trim_end_matches(",").to_string();
                }
                sql_create = sql_create + &format!("({});", sql_column);
                let result_create = rb.exec(&sql_create, vec![]).await;
                match result_create {
                    Ok(_) => {}
                    Err(e) => {
                        if e.to_string().contains("already exists") {
                            for (k, v) in &m {
                                let k = k.as_str().unwrap_or_default();
                                let mut id_key = "";
                                if k.eq("id") || v.as_str().unwrap_or_default() == "id" {
                                    id_key = " PRIMARY KEY NOT NULL";
                                }
                                match rb.exec(&format!("alter table {} add {} {} {};", name, k, type_str(&v), id_key), vec![]).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        if e.to_string().contains("duplicate column") {
                                            continue;
                                        }
                                        return Err(e);
                                    }
                                }
                            }
                            return Ok(());
                        }
                        return Err(e);
                    }
                }
                Ok(())
            }
            Value::Ext(table_name, m) => {
                self.sync(rb, *m, name).await
            }
            _ => {
                Err(Error::from("table not is an struct or map!"))
            }
        }
    }
}