use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::rbatis::Rbatis;
use crate::snowflake::new_snowflake_id;
use crate::sql::page::{IPageRequest, Page};
use crate::sql::tx::Tx;
use crate::utils::string_util;
use async_trait::async_trait;
use flume::RecvError;
use futures::executor::block_on;
use futures::Future;
use futures_core::future::BoxFuture;
use crate::decode::decode;
use rbdc::db::{Connection, ExecResult};
use rbs::{from_value, Value};
use serde::de::DeserializeOwned;
use serde::{Serialize, Serializer};
use crate::Error;

/// the rbatis's Executor. this trait maybe is tx,connection,rbatis object
#[async_trait]
pub trait Executor: RbatisRef {
    async fn exec(&mut self, sql: &str, args: Vec<Value>) -> Result<ExecResult, Error>;
    async fn fetch(&mut self, sql: &str, args: Vec<Value>) -> Result<Value, Error>;
}

#[async_trait]
pub trait RbatisRef: Send {
    fn get_rbatis(&self) -> &Rbatis;

    fn driver_type(&self) -> crate::Result<&str> {
        self.get_rbatis().driver_type()
    }
}

impl RbatisRef for Rbatis {
    fn get_rbatis(&self) -> &Rbatis {
        &self
    }
}

pub struct RBatisConnExecutor {
    pub conn: Box<dyn Connection>,
    pub rb: Rbatis,
}

impl Debug for RBatisConnExecutor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.rb.fmt(f)
    }
}

impl RBatisConnExecutor {
    pub async fn exec(&mut self, sql: &str, args: Vec<Value>) -> Result<ExecResult, Error>
    {
        let v = Executor::exec(self, sql, args).await?;
        Ok(v)
    }

    pub async fn fetch(&mut self, sql: &str, args: Vec<Value>) -> Result<Value, Error>
    {
        let v = Executor::fetch(self, sql, args).await?;
        Ok(v)
    }

    pub async fn fetch_decode<T>(&mut self, sql: &str, args: Vec<Value>) -> Result<T, Error>
        where
            T: DeserializeOwned,
    {
        let v = Executor::fetch(self, sql, args).await?;
        Ok(decode(v)?)
    }
}

fn arr_to_string(arg: Vec<Value>) -> (Vec<Value>, String) {
    let b = Value::Array(arg);
    let s = b.to_string();
    return match b {
        Value::Array(arr) => (arr, s),
        _ => (vec![], s),
    };
}

#[async_trait]
impl Executor for RBatisConnExecutor {
    async fn exec(&mut self, sql: &str, mut args: Vec<Value>) -> Result<ExecResult, Error> {
        let rb_task_id = new_snowflake_id();
        let mut sql = sql.to_string();
        let is_prepared = args.len() > 0;
        for item in self.get_rbatis().sql_intercepts.iter() {
            item.do_intercept(self.get_rbatis(), &mut sql, &mut args, is_prepared)?;
        }
        if self.get_rbatis().log_plugin.is_enable() {
            let (_args, args_string) = arr_to_string(args);
            args = _args;
            self.get_rbatis().log_plugin.info(
                rb_task_id,
                &format!(
                    "Exec   ==> {}\n{}[rbatis]                      Args   ==> {}",
                    &sql,
                    string_util::LOG_SPACE,
                    args_string
                ),
            );
        }
        let result = self.conn.exec(&sql, args).await;
        if self.get_rbatis().log_plugin.is_enable() {
            match &result {
                Ok(result) => {
                    self.get_rbatis().log_plugin.info(
                        rb_task_id,
                        &format!("RowsAffected <== {}", result.rows_affected),
                    );
                }
                Err(e) => {
                    self.get_rbatis()
                        .log_plugin
                        .error(rb_task_id, &format!("ReturnErr  <== {}", e));
                }
            }
        }
        return result;
    }

    async fn fetch(&mut self, sql: &str, mut args: Vec<Value>) -> Result<Value, Error> {
        let rb_task_id = new_snowflake_id();
        let mut sql = sql.to_string();
        let is_prepared = args.len() > 0;
        for item in self.get_rbatis().sql_intercepts.iter() {
            item.do_intercept(self.get_rbatis(), &mut sql, &mut args, is_prepared)?;
        }
        if self.get_rbatis().log_plugin.is_enable() {
            let (_args, args_string) = arr_to_string(args);
            args = _args;
            self.get_rbatis().log_plugin.info(
                rb_task_id,
                &format!(
                    "Fetch  ==> {}\n{}[rbatis]                      Args   ==> {}",
                    &sql,
                    string_util::LOG_SPACE,
                    args_string
                ),
            );
        }
        let result = self.conn.get_values(&sql, args).await;
        if self.get_rbatis().log_plugin.is_enable() {
            match &result {
                Ok(result) => {
                    self.get_rbatis()
                        .log_plugin
                        .info(rb_task_id, &format!("ReturnRows <== {:?}", result));
                }
                Err(e) => {
                    self.get_rbatis()
                        .log_plugin
                        .error(rb_task_id, &format!("ReturnErr  <== {}", e));
                }
            }
        }
        return Ok(Value::Array(result?));
    }
}

impl RbatisRef for RBatisConnExecutor {
    fn get_rbatis(&self) -> &Rbatis {
        &self.rb
    }
}

impl RBatisConnExecutor {
    pub async fn begin(self) -> crate::Result<RBatisTxExecutor> {
        let tx = self.conn.begin().await?;
        return Ok(RBatisTxExecutor {
            tx_id: new_snowflake_id(),
            conn: tx,
            rb: self.rb,
            done: false,
        });
    }
}

pub struct RBatisTxExecutor {
    pub tx_id: i64,
    pub conn: Box<dyn Connection>,
    pub rb: Rbatis,
    pub done: bool,
}

impl Debug for RBatisTxExecutor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RBatisTxExecutor")
            .field("tx_id", &self.tx_id)
            .field("rb", &self.rb)
            .finish()
    }
}

impl<'a> RBatisTxExecutor {
    /// exec
    pub async fn exec(&mut self, sql: &str, args: Vec<Value>) -> Result<ExecResult, Error>
    {
        let v = Executor::exec(self, sql, args).await?;
        Ok(v)
    }
    /// fetch value
    pub async fn fetch(&mut self, sql: &str, args: Vec<Value>) -> Result<Value, Error>
    {
        let v = Executor::fetch(self, sql, args).await?;
        Ok(v)
    }
    /// fetch and decode
    pub async fn fetch_decode<T>(&mut self, sql: &str, args: Vec<Value>) -> Result<Value, Error>
        where
            T: DeserializeOwned,
    {
        let v = Executor::fetch(self, sql, args).await?;
        Ok(decode(v)?)
    }
}

#[async_trait]
impl Executor for RBatisTxExecutor {
    async fn exec(
        &mut self,
        sql: &str,
        mut args: Vec<Value>,
    ) -> Result<rbdc::db::ExecResult, Error> {
        let mut sql = sql.to_string();
        let is_prepared = args.len() > 0;
        for item in self.get_rbatis().sql_intercepts.iter() {
            item.do_intercept(self.get_rbatis(), &mut sql, &mut args, is_prepared)?;
        }
        if self.get_rbatis().log_plugin.is_enable() {
            let (_args, args_string) = arr_to_string(args);
            args = _args;
            self.get_rbatis().log_plugin.info(
                self.tx_id,
                &format!(
                    "Exec   ==> {}\n{}[rbatis]                      Args   ==> {}",
                    &sql,
                    string_util::LOG_SPACE,
                    args_string
                ),
            );
        }
        let result = self.conn.exec(&sql, args).await;
        if self.get_rbatis().log_plugin.is_enable() {
            match &result {
                Ok(result) => {
                    self.get_rbatis().log_plugin.info(
                        self.tx_id,
                        &format!("RowsAffected <== {}", result.rows_affected),
                    );
                }
                Err(e) => {
                    self.get_rbatis()
                        .log_plugin
                        .error(self.tx_id, &format!("ReturnErr  <== {}", e));
                }
            }
        }
        return result;
    }

    async fn fetch(&mut self, sql: &str, mut args: Vec<Value>) -> Result<Value, Error> {
        let mut sql = sql.to_string();
        let is_prepared = args.len() > 0;
        for item in self.get_rbatis().sql_intercepts.iter() {
            item.do_intercept(self.get_rbatis(), &mut sql, &mut args, is_prepared)?;
        }
        if self.get_rbatis().log_plugin.is_enable() {
            let (_args, args_string) = arr_to_string(args);
            args = _args;
            self.get_rbatis().log_plugin.info(
                self.tx_id,
                &format!(
                    "Fetch  ==> {}\n{}[rbatis]                      Args   ==> {}",
                    &sql,
                    string_util::LOG_SPACE,
                    args_string
                ),
            );
        }
        let result = self.conn.get_values(&sql, args).await;
        if self.get_rbatis().log_plugin.is_enable() {
            match &result {
                Ok(result) => {
                    self.get_rbatis()
                        .log_plugin
                        .info(self.tx_id, &format!("ReturnRows <== {:?}", result));
                }
                Err(e) => {
                    self.get_rbatis()
                        .log_plugin
                        .error(self.tx_id, &format!("ReturnErr  <== {}", e));
                }
            }
        }
        return Ok(Value::Array(result?));
    }
}

impl RbatisRef for RBatisTxExecutor {
    fn get_rbatis(&self) -> &Rbatis {
        &self.rb
    }
}

impl RBatisTxExecutor {
    pub async fn begin(mut self) -> crate::Result<Self> {
        self.conn = self.conn.begin().await?;
        return Ok(self);
    }
    pub async fn commit(&mut self) -> crate::Result<bool> {
        if let Ok(()) = self.conn.commit().await {
            self.done = true;
        }
        return Ok(self.done);
    }
    pub async fn rollback(&mut self) -> crate::Result<bool> {
        if let Ok(()) = self.conn.rollback().await {
            self.done = true;
        }
        return Ok(self.done);
    }

    pub fn take_conn(self) -> Box<dyn Connection> {
        return self.conn;
    }
}

impl Deref for RBatisTxExecutor {
    type Target = Box<dyn Connection>;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl DerefMut for RBatisTxExecutor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

pub struct RBatisTxExecutorGuard {
    pub tx: Option<RBatisTxExecutor>,
    pub sender: flume::Sender<RBatisTxExecutor>,
}

impl Debug for RBatisTxExecutorGuard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RBatisTxExecutorGuard")
            .field("tx", &self.tx)
            .finish()
    }
}

impl RBatisTxExecutorGuard {
    pub async fn begin(&mut self) -> crate::Result<()> {
        let v = self
            .tx
            .take()
            .ok_or_else(|| Error::from("[rbatis] tx is committed"))?
            .begin()
            .await?;
        self.tx = Some(v);
        return Ok(());
    }

    pub async fn commit(&mut self) -> crate::Result<bool> {
        let tx = self
            .tx
            .as_mut()
            .ok_or_else(|| Error::from("[rbatis] tx is committed"))?;
        return Ok(tx.commit().await?);
    }

    pub async fn rollback(&mut self) -> crate::Result<bool> {
        let tx = self
            .tx
            .as_mut()
            .ok_or_else(|| Error::from("[rbatis] tx is committed"))?;
        return Ok(tx.rollback().await?);
    }

    pub fn take_conn(mut self) -> Option<Box<dyn Connection>> {
        match self.tx.take() {
            None => None,
            Some(s) => s.take_conn().into(),
        }
    }
}

impl RBatisTxExecutor {
    /// defer and use future method
    /// for example:
    ///         tx.defer_async(|mut tx| async {
    ///             tx.rollback().await;
    ///         });
    ///
    pub fn defer_async<R, F>(self, mut callback: F) -> RBatisTxExecutorGuard
        where
            R: Future<Output=()> + 'static + Send,
            F: Send + FnMut(RBatisTxExecutor) -> R + 'static,
    {
        let (s, r) = flume::unbounded();
        rbdc::rt::spawn(async move {
            match r.recv_async().await {
                Ok(v) => {
                    callback(v).await;
                }
                Err(e) => {
                    log::error!("rbatis recv_async tx fail:{}",e);
                }
            }
        });
        RBatisTxExecutorGuard {
            tx: Some(self),
            sender: s,
        }
    }
}

impl Deref for RBatisTxExecutorGuard {
    type Target = RBatisTxExecutor;

    fn deref(&self) -> &Self::Target {
        &self.tx.as_ref().unwrap()
    }
}

impl<'a> DerefMut for RBatisTxExecutorGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tx.as_mut().unwrap()
    }
}

impl Drop for RBatisTxExecutorGuard {
    fn drop(&mut self) {
        match self.tx.take() {
            None => {}
            Some(tx) => {
                if let Err(e) = self.sender.send(tx) {
                    log::error!("rbatis send tx fail: {}",e);
                }
            }
        }
    }
}

impl RbatisRef for RBatisTxExecutorGuard {
    fn get_rbatis(&self) -> &Rbatis {
        &self.rb
    }
}

#[async_trait]
impl Executor for RBatisTxExecutorGuard{
    async fn exec(&mut self, sql: &str, args: Vec<Value>) -> Result<ExecResult, Error> {
        match self.tx.as_mut(){
            None => {
                Err(Error::from("the tx is done!"))
            }
            Some(v) => {
                v.exec(sql,args).await
            }
        }
    }

    async fn fetch(&mut self, sql: &str, args: Vec<Value>) -> Result<Value, Error> {
        match self.tx.as_mut(){
            None => {
                Err(Error::from("the tx is done!"))
            }
            Some(v) => {
                v.fetch(sql,args).await
            }
        }
    }
}

impl Rbatis {
    /// exec sql
    pub async fn exec(&self, sql: &str, args: Vec<Value>) -> Result<rbdc::db::ExecResult, Error> {
        let mut conn = self.acquire().await?;
        conn.exec(sql, args).await
    }

    /// fetch raw Value
    pub async fn fetch(&self, sql: &str, args: Vec<Value>) -> Result<Value, Error> {
        let mut conn = self.acquire().await?;
        let v = conn.fetch(sql, args).await?;
        Ok(v)
    }

    /// fetch and decode
    pub async fn fetch_decode<T>(&self, sql: &str, args: Vec<Value>) -> Result<T, Error>
        where
            T: DeserializeOwned,
    {
        let mut conn = self.acquire().await?;
        let v = conn.fetch(sql, args).await?;
        Ok(decode(v)?)
    }
}

#[async_trait]
impl Executor for Rbatis {
    async fn exec(&mut self, sql: &str, args: Vec<Value>) -> Result<rbdc::db::ExecResult, Error> {
        let mut conn = self.acquire().await?;
        conn.exec(sql, args).await
    }

    async fn fetch(&mut self, sql: &str, args: Vec<Value>) -> Result<Value, Error> {
        let mut conn = self.acquire().await?;
        conn.fetch(sql, args).await
    }
}

impl RbatisRef for &Rbatis {
    fn get_rbatis(&self) -> &Rbatis {
        self
    }
}

#[async_trait]
impl Executor for &Rbatis {
    async fn exec(&mut self, sql: &str, args: Vec<Value>) -> Result<rbdc::db::ExecResult, Error> {
        let mut conn = self.acquire().await?;
        conn.exec(sql, args).await
    }

    async fn fetch(&mut self, sql: &str, args: Vec<Value>) -> Result<Value, Error> {
        let mut conn = self.acquire().await?;
        conn.fetch(sql, args).await
    }
}
