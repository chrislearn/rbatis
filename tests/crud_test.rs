#[macro_use]
extern crate rbatis;

#[cfg(test)]
mod test {
    use std::any::Any;
    use futures_core::future::BoxFuture;
    use rbatis::{Error, Rbatis};
    use rbatis::sql::PageRequest;
    use rbdc::block_on;
    use rbdc::datetime::FastDateTime;
    use rbdc::db::{Connection, ConnectOptions, Driver, ExecResult, MetaData, Row};
    use rbdc::rt::block_on;
    use rbs::Value;

    #[derive(Debug, Clone)]
    pub struct MockDriver {}

    impl Driver for MockDriver {
        fn name(&self) -> &str {
            "test"
        }

        fn connect(&self, url: &str) -> BoxFuture<Result<Box<dyn Connection>, Error>> {
            Box::pin(async {
                Ok(Box::new(MockConnection {}) as Box<dyn Connection>)
            })
        }

        fn connect_opt<'a>(&'a self, opt: &'a dyn ConnectOptions) -> BoxFuture<Result<Box<dyn Connection>, Error>> {
            Box::pin(async {
                Ok(Box::new(MockConnection {}) as Box<dyn Connection>)
            })
        }

        fn default_option(&self) -> Box<dyn ConnectOptions> {
            Box::new(MockConnectOptions {})
        }
    }

    #[derive(Clone, Debug)]
    pub struct MockRowMetaData {
        sql: String,
    }

    impl MetaData for MockRowMetaData {
        fn column_len(&self) -> usize {
            if self.sql.contains("select count") {
                1
            } else {
                2
            }
        }

        fn column_name(&self, i: usize) -> String {
            if self.sql.contains("select count") {
                "count".to_string()
            } else {
                if i == 0 {
                    "sql".to_string()
                } else {
                    "count".to_string()
                }
            }
        }

        fn column_type(&self, i: usize) -> String {
            "String".to_string()
        }
    }

    #[derive(Clone, Debug)]
    pub struct MockRow {
        pub sql: String,
        pub count: u64,
    }

    impl Row for MockRow {
        fn meta_data(&self) -> Box<dyn MetaData> {
            Box::new(MockRowMetaData {
                sql: self.sql.clone()
            }) as Box<dyn MetaData>
        }

        fn get(&mut self, i: usize) -> Option<Value> {
            if self.sql.contains("select count") {
                Some(Value::U64(self.count))
            } else {
                if i == 0 {
                    Some(Value::String(self.sql.clone()))
                } else {
                    Some(Value::U64(self.count.clone()))
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct MockConnection {}

    impl Connection for MockConnection {
        fn get_rows(&mut self, sql: &str, params: Vec<Value>) -> BoxFuture<Result<Vec<Box<dyn Row>>, Error>> {
            let sql = sql.to_string();
            Box::pin(async move {
                let data = Box::new(MockRow {
                    sql: sql,
                    count: 1,
                }) as Box<dyn Row>;
                Ok(vec![data])
            })
        }

        fn exec(&mut self, sql: &str, params: Vec<Value>) -> BoxFuture<Result<ExecResult, Error>> {
            let sql = sql.to_string();
            Box::pin(async move {
                Ok(ExecResult {
                    rows_affected: 0,
                    last_insert_id: Value::String(sql.to_string()),
                })
            })
        }

        fn close(&mut self) -> BoxFuture<Result<(), Error>> {
            Box::pin(async {
                Ok(())
            })
        }

        fn ping(&mut self) -> BoxFuture<Result<(), Error>> {
            Box::pin(async {
                Ok(())
            })
        }
    }

    #[derive(Clone, Debug)]
    pub struct MockConnectOptions {}

    impl ConnectOptions for MockConnectOptions {
        fn connect(&self) -> BoxFuture<Result<Box<dyn Connection>, Error>> {
            Box::pin(async {
                Ok(Box::new(MockConnection {}) as Box<dyn Connection>)
            })
        }

        fn set_uri(&mut self, uri: &str) -> Result<(), Error> {
            Ok(())
        }

        fn uppercase_self(&self) -> &(dyn Any + Send + Sync) {
            self
        }
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct MockTable {
        pub id: Option<String>,
        pub name: Option<String>,
        pub pc_link: Option<String>,
        pub h5_link: Option<String>,
        pub pc_banner_img: Option<String>,
        pub h5_banner_img: Option<String>,
        pub sort: Option<String>,
        pub status: Option<i32>,
        pub remark: Option<String>,
        pub create_time: Option<rbdc::datetime::FastDateTime>,
        pub version: Option<i64>,
        pub delete_flag: Option<i32>,

        pub sql: String,
        //exec sql
        pub count: u64,//page count num
    }
    crud!(MockTable {});
    #[test]
    fn test_insert() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::insert(&mut rb, &t).await.unwrap();
            println!("{}", r.last_insert_id.as_str().unwrap_or_default());
            assert_eq!(r.last_insert_id.as_str().unwrap_or_default(), "insert into mock_table (id,name,pc_link,h5_link,pc_banner_img,h5_banner_img,sort,status,remark,create_time,version,delete_flag,sql,count) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)");
        };
        block_on(f);
    }

    #[test]
    fn test_insert_batch() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let mut t2 = t.clone();
            t2.id = "3".to_string().into();
            let r = MockTable::insert_batch(&mut rb, &[t, t2]).await.unwrap();
            println!("{}", r.last_insert_id.as_str().unwrap_or_default());
            assert_eq!(r.last_insert_id.as_str().unwrap_or_default(), "insert into mock_table (id,name,pc_link,h5_link,pc_banner_img,h5_banner_img,sort,status,remark,create_time,version,delete_flag,sql,count) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?),(?,?,?,?,?,?,?,?,?,?,?,?,?,?)");
        };
        block_on(f);
    }

    #[test]
    fn test_update_by_column() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::update_by_column(&mut rb, &t, "id").await.unwrap();
            println!("{}", r.last_insert_id.as_str().unwrap_or_default());
            assert_eq!(r.last_insert_id.as_str().unwrap_or_default(), "update mock_table set name=?,pc_link=?,h5_link=?,status=?,remark=?,create_time=?,version=?,delete_flag=?,sql=?,count=? where  id = ?");
        };
        block_on(f);
    }

    #[test]
    fn test_select_all() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::select_all(&mut rb).await.unwrap();
            println!("{:?}", r[0].sql);
            assert_eq!(r[0].sql, "select * from mock_table");
        };
        block_on(f);
    }

    #[test]
    fn test_delete_by_column() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let r = MockTable::delete_by_column(&mut rb, "1", &Value::String("1".to_string())).await.unwrap();
            println!("{}", r.last_insert_id.as_str().unwrap_or_default());
            assert_eq!(r.last_insert_id.as_str().unwrap_or_default(), "delete from mock_table where  1 = ?");
        };
        block_on(f);
    }

    #[test]
    fn test_delete_by_column_batch() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let r = MockTable::delete_by_column_batch(&mut rb, "1", &["1", "2"]).await.unwrap();
            println!("{}", r.last_insert_id.as_str().unwrap_or_default());
            assert_eq!(r.last_insert_id.as_str().unwrap_or_default(), "delete from mock_table where  1 in (?,?)");
        };
        block_on(f);
    }

    impl_select!(MockTable{select_all_by_id(id:&str,name:&str) => "`where id = #{id} and name = #{name}`"});
    #[test]
    fn test_select_all_by_id() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::select_all_by_id(&mut rb, "1", "1").await.unwrap();
            println!("{}", r[0].sql);
            assert_eq!(r[0].sql, "select * from mock_table where id = ? and name = ?");
        };
        block_on(f);
    }
    impl_select!(MockTable{select_by_id(id:&str) -> Option => "`where id = #{id} limit 1`"});
    #[test]
    fn test_select_by_id() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::select_by_id(&mut rb, "1").await.unwrap();
            println!("{}", r.as_ref().unwrap().sql);
            assert_eq!(r.unwrap().sql, "select * from mock_table where id = ? limit 1");
        };
        block_on(f);
    }
    impl_update!(MockTable{update_by_name(name:&str) => "`where id = '2'`"});
    #[test]
    fn test_update_by_name() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::update_by_name(&mut rb, &t, "test").await.unwrap();
            println!("{}", r.last_insert_id.as_str().unwrap());
            assert_eq!(r.last_insert_id.as_str().unwrap(), "update mock_table set  id=?,name=?,pc_link=?,h5_link=?,status=?,remark=?,create_time=?,version=?,delete_flag=?,sql=?,count=? where id = '2'");
        };
        block_on(f);
    }
    impl_delete!(MockTable {delete_by_name(name:&str) => "`where name= '2'`"});
    #[test]
    fn test_delete_by_name() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::delete_by_name(&mut rb, "2").await.unwrap();
            println!("{}", r.last_insert_id.as_str().unwrap());
            assert_eq!(r.last_insert_id.as_str().unwrap(), "delete from mock_table where name= '2'");
        };
        block_on(f);
    }
    impl_select_page!(MockTable{select_page() => "`order by create_time desc`"});
    #[test]
    fn test_select_page() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::select_page(&mut rb, &PageRequest::new(1, 10)).await.unwrap();
            println!("{}", r.records[0].sql);
            assert_eq!(r.records[0].sql, "select * from mock_table order by create_time desc limit 0,10");
        };
        block_on(f);
    }
    impl_select_page!(MockTable{select_page_by_name(name:&str) =>"
     if name != null && name != '':
       `where name != #{name}`
     if name == '':
       `where name != ''`"});
    #[test]
    fn test_select_page_by_name() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::select_page_by_name(&mut rb, &PageRequest::new(1, 10), "").await.unwrap();
            println!("{}", r.records[0].sql);
            assert_eq!(r.records[0].sql, "select * from mock_table where name != '' limit 0,10");
        };
        block_on(f);
    }

    #[test]
    fn test_select_by_column() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let r = MockTable::select_by_column(&mut rb, "id","1").await.unwrap();
            println!("{}", r[0].sql);
            assert_eq!(r[0].sql, "select * from mock_table where id = ?");
        };
        block_on(f);
    }

    #[test]
    fn test_tx() {
        let f = async move {
            let mut rb = Rbatis::new();
            rb.link(MockDriver {}, "test").await.unwrap();
            let mut t = MockTable {
                id: Some("2".into()),
                name: Some("2".into()),
                pc_link: Some("2".into()),
                h5_link: Some("2".into()),
                pc_banner_img: None,
                h5_banner_img: None,
                sort: None,
                status: Some(2),
                remark: Some("2".into()),
                create_time: Some(FastDateTime::now()),
                version: Some(1),
                sql: "".to_string(),
                delete_flag: Some(1),
                count: 0,
            };
            let mut tx = rb.acquire_begin().await.unwrap();
            let r = MockTable::insert(&mut tx, &t).await.unwrap();

            let mut tx = rb.acquire_begin().await.unwrap().defer_async(|tx| async {});
            let r = MockTable::insert(&mut tx, &t).await.unwrap();
        };
        block_on(f);
    }
}