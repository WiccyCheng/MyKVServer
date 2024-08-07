pub mod abi;

use abi::{command_request::RequestData, *};
use bytes::Bytes;
use http::StatusCode;
use prost::Message;
use std::fmt::Display;

use crate::KvError;

impl CommandRequest {
    /// 创建 HGET 命令
    pub fn new_hget(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hget(Hget {
                table: table.into(),
                key: key.into(),
            })),
        }
    }

    /// 创建 HGETALL 命令
    pub fn new_hgetall(table: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hgetall(Hgetall {
                table: table.into(),
            })),
        }
    }

    /// 创建 HSET 命令
    pub fn new_hset(
        table: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<Value>,
    ) -> Self {
        Self {
            request_data: Some(RequestData::Hset(Hset {
                table: table.into(),
                pair: Some(Kvpair::new(key, value)),
            })),
        }
    }

    /// 创建 HDEL 命令
    pub fn new_hdel(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hdel(Hdel {
                table: table.into(),
                key: key.into(),
            })),
        }
    }

    /// 创建 HEXIST 命令
    pub fn new_hexist(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hexist(Hexist {
                table: table.into(),
                key: key.into(),
            })),
        }
    }

    /// 创建 HMGET 命令
    pub fn new_hmget(table: impl Into<String>, keys: Vec<impl Into<String>>) -> Self {
        Self {
            request_data: Some(RequestData::Hmget(Hmget {
                table: table.into(),
                keys: keys.into_iter().map(|key| key.into()).collect(),
            })),
        }
    }

    /// 创建 HMSET 命令
    pub fn new_hmset(table: impl Into<String>, pairs: Vec<impl Into<Kvpair>>) -> Self {
        Self {
            request_data: Some(RequestData::Hmset(Hmset {
                table: table.into(),
                pairs: pairs.into_iter().map(|pair| pair.into()).collect(),
            })),
        }
    }
    /// 创建 HMDEL 命令
    pub fn new_hmdel(table: impl Into<String>, keys: Vec<impl Into<String>>) -> Self {
        Self {
            request_data: Some(RequestData::Hmdel(Hmdel {
                table: table.into(),
                keys: keys.into_iter().map(|key| key.into()).collect(),
            })),
        }
    }

    /// 创建 HMEXIST 命令
    pub fn new_hmexist(table: impl Into<String>, keys: Vec<impl Into<String>>) -> Self {
        Self {
            request_data: Some(RequestData::Hmexist(Hmexist {
                table: table.into(),
                keys: keys.into_iter().map(|key| key.into()).collect(),
            })),
        }
    }

    /// 创建 SUBSCRIBE 命令
    pub fn new_subscribe(name: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Subscribe(Subscribe { topic: name.into() })),
        }
    }

    /// 创建 UNSUBSCRIBE 命令
    pub fn new_unsubscribe(name: impl Into<String>, id: u32) -> Self {
        Self {
            request_data: Some(RequestData::Unsubscribe(Unsubscribe {
                topic: name.into(),
                id,
            })),
        }
    }

    /// 创建 PUBLISH 命令
    pub fn new_publish(name: impl Into<String>, data: Vec<Value>) -> Self {
        Self {
            request_data: Some(RequestData::Publish(Publish {
                topic: name.into(),
                data,
            })),
        }
    }

    /// 转换成 string 做错误处理
    pub fn format(&self) -> String {
        format!("{:?}", self)
    }
}

impl CommandResponse {
    pub fn ok() -> Self {
        CommandResponse {
            status: StatusCode::OK.as_u16() as _,
            ..Default::default()
        }
    }

    pub fn internal_error(msg: String) -> Self {
        CommandResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as _,
            message: msg,
            ..Default::default()
        }
    }

    /// 转换成 string 做错误处理
    pub fn format(&self) -> String {
        format!("{:?}", self)
    }
}

impl Value {
    /// 转换成 string 做错误处理
    pub fn format(&self) -> String {
        format!("{:?}", self)
    }
}

impl Kvpair {
    // 创建一个新的 kv pair
    pub fn new(key: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            key: key.into(),
            value: Some(value.into()),
        }
    }
}

/// 从(String, Value)转成Kvpair
impl From<(String, Value)> for Kvpair {
    fn from(data: (String, Value)) -> Self {
        Kvpair::new(data.0, data.1)
    }
}

/// 从bool转成Value
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self {
            value: Some(value::Value::Bool(v)),
        }
    }
}

/// 从String转成Value
impl From<String> for Value {
    fn from(s: String) -> Self {
        Self {
            value: Some(value::Value::String(s)),
        }
    }
}

/// 从&str转成Value
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self {
            value: Some(value::Value::String(s.into())),
        }
    }
}

/// 从i64转成Value
impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self {
            value: Some(value::Value::Integer(i)),
        }
    }
}

/// 从Value转换成CommandResponse
impl From<Value> for CommandResponse {
    fn from(v: Value) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: vec![v],
            ..Default::default()
        }
    }
}

/// 从Vec<Kvpair> 转换成CommandResponse
impl From<Vec<Kvpair>> for CommandResponse {
    fn from(v: Vec<Kvpair>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            pairs: v,
            ..Default::default()
        }
    }
}

/// 从KvError 转换成 CommandResponse
impl From<KvError> for CommandResponse {
    fn from(e: KvError) -> Self {
        let mut result = Self {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as _,
            message: e.to_string(),
            values: vec![],
            pairs: vec![],
        };

        match e {
            KvError::NotFound(_) => result.status = StatusCode::NOT_FOUND.as_u16() as _,
            KvError::InvalidCommand(_) => result.status = StatusCode::BAD_REQUEST.as_u16() as _,
            _ => {}
        };

        result
    }
}

/// 从Vec<Value> 转换成 CommandResponse
impl From<Vec<Value>> for CommandResponse {
    fn from(v: Vec<Value>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: v,
            ..Default::default()
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Integer(i)) => Ok(i),
            _ => Err(KvError::ConvertError(v.format(), "Integer")),
        }
    }
}

impl TryFrom<&Value> for i64 {
    type Error = KvError;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Integer(i)) => Ok(i),
            _ => Err(KvError::ConvertError(v.format(), "Integer")),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Float(f)) => Ok(f),
            _ => Err(KvError::ConvertError(v.format(), "Float")),
        }
    }
}

impl TryFrom<Value> for Bytes {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Binary(b)) => Ok(b),
            _ => Err(KvError::ConvertError(v.format(), "Binary")),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Bool(b)) => Ok(b),
            _ => Err(KvError::ConvertError(v.format(), "Bool")),
        }
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        let mut buf = Vec::with_capacity(v.encoded_len());
        v.encode(&mut buf)?;
        Ok(buf)
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = KvError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let msg = Value::decode(data)?;
        Ok(msg)
    }
}

impl From<(Box<[u8]>, Box<[u8]>)> for Kvpair {
    fn from(pair: (Box<[u8]>, Box<[u8]>)) -> Self {
        Kvpair::new(
            std::str::from_utf8(pair.0.as_ref()).unwrap(),
            TryInto::<Value>::try_into(pair.1.as_ref()).unwrap(),
        )
    }
}

impl<const N: usize> From<&[u8; N]> for Value {
    fn from(buf: &[u8; N]) -> Self {
        Bytes::copy_from_slice(&buf[..]).into()
    }
}

impl From<Bytes> for Value {
    fn from(buf: Bytes) -> Self {
        Self {
            value: Some(value::Value::Binary(buf)),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(value) => write!(f, "{:?}", value),
            None => Ok({}),
        }
    }
}

impl Display for Kvpair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.value.is_some() {
            return write!(
                f,
                "key: {}, value: {}",
                self.key,
                self.value.as_ref().unwrap(),
            );
        } else {
            return write!(f, "key: {}, value: None", self.key,);
        }
    }
}

impl Display for CommandResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status: {}\n", self.status)?;

        if !self.message.is_empty() {
            writeln!(f, "Message: {}", self.message)?;
        }

        if !self.values.is_empty() {
            writeln!(f, "Values:")?;
            for value in &self.values {
                writeln!(f, "  {}", value)?;
            }
        }

        if !self.pairs.is_empty() {
            writeln!(f, "Pairs:")?;
            for pair in &self.pairs {
                writeln!(f, "  {}", pair)?;
            }
        }

        Ok(())
    }
}

impl TryFrom<&CommandResponse> for i64 {
    type Error = KvError;

    fn try_from(value: &CommandResponse) -> Result<Self, Self::Error> {
        if value.status != StatusCode::OK.as_u16() as u32 {
            return Err(KvError::ConvertError(value.format(), "CommandResponse"));
        } else {
            match value.values.get(0) {
                Some(v) => v.try_into(),
                None => Err(KvError::ConvertError(value.format(), "CommandResponse")),
            }
        }
    }
}
