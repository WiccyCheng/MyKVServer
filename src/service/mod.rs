mod command_service;
mod topic;
mod topic_service;

pub use topic::{Broadcaster, Topic};
use topic_service::{StreamingResponse, TopicService};

use futures::stream;
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::{
    command_request::RequestData, CommandRequest, CommandResponse, KvError, MemTable, Storage,
};

/// 对command的处理的抽象
pub trait CommandService {
    // 处理 Command，返回 Response
    fn execute(self, store: &impl Storage) -> CommandResponse;
}

/// 事件通知（不可变事件）
pub trait Notify<Arg> {
    fn notify(&self, arg: &Arg);
}

/// 事件通知（可变事件）
pub trait NotifyMut<Arg> {
    fn notify(&self, arg: &mut Arg);
}

impl<Arg> Notify<Arg> for Vec<fn(&Arg)> {
    #[inline]
    fn notify(&self, arg: &Arg) {
        for f in self {
            f(arg)
        }
    }
}

impl<Arg> NotifyMut<Arg> for Vec<fn(&mut Arg)> {
    #[inline]
    fn notify(&self, arg: &mut Arg) {
        for f in self {
            f(arg)
        }
    }
}

/// Service 数据结构
pub struct Service<Store = MemTable> {
    // TODO(Wiccy): 通过对key做哈希映射将操作分散到多个线程各自持有的HashMap中，避免加锁
    inner: Arc<ServiceInner<Store>>,
    broadcaster: Arc<Broadcaster>,
}

impl<Store> Clone for Service<Store> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            broadcaster: Arc::clone(&self.broadcaster),
        }
    }
}

impl<Store: Storage> Service<Store> {
    #[instrument(name = "service_execute", skip_all)]
    pub fn execute(&self, cmd: CommandRequest) -> StreamingResponse {
        debug!("Got request: {:?}", cmd);
        self.inner.on_received.notify(&cmd);
        let mut res = dispatch(cmd.clone(), &self.inner.store);

        if res == CommandResponse::default() {
            dispatch_stream(cmd, Arc::clone(&self.broadcaster))
        } else {
            debug!("Executed response: {:?}", res);
            self.inner.on_executed.notify(&res);
            self.inner.on_before_send.notify(&mut res);
            if !self.inner.on_before_send.is_empty() {
                debug!("Modified response: {:?}", res);
            }
            Box::pin(stream::once(async { Arc::new(res) }))
        }
    }
}

/// Service 内部数据结构
pub struct ServiceInner<Store> {
    store: Store,
    on_received: Vec<fn(&CommandRequest)>,
    on_executed: Vec<fn(&CommandResponse)>,
    on_before_send: Vec<fn(&mut CommandResponse)>,
    on_after_send: Vec<fn()>,
}

impl<Store: Storage> ServiceInner<Store> {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            on_received: Vec::new(),
            on_executed: Vec::new(),
            on_before_send: Vec::new(),
            on_after_send: Vec::new(),
        }
    }

    pub fn fn_received(mut self, f: fn(&CommandRequest)) -> Self {
        self.on_received.push(f);
        self
    }
    pub fn fn_executed(mut self, f: fn(&CommandResponse)) -> Self {
        self.on_executed.push(f);
        self
    }
    pub fn fn_before_send(mut self, f: fn(&mut CommandResponse)) -> Self {
        self.on_before_send.push(f);
        self
    }
    pub fn fn_after_send(mut self, f: fn()) -> Self {
        self.on_after_send.push(f);
        self
    }
}

impl<Store: Storage> From<ServiceInner<Store>> for Service<Store> {
    fn from(inner: ServiceInner<Store>) -> Self {
        Service {
            inner: Arc::new(inner),
            broadcaster: Default::default(),
        }
    }
}

pub fn dispatch(cmd: CommandRequest, store: &impl Storage) -> CommandResponse {
    match cmd.request_data {
        Some(RequestData::Hget(param)) => param.execute(store),
        Some(RequestData::Hset(param)) => param.execute(store),
        Some(RequestData::Hdel(param)) => param.execute(store),
        Some(RequestData::Hexist(param)) => param.execute(store),
        Some(RequestData::Hmget(param)) => param.execute(store),
        Some(RequestData::Hmset(param)) => param.execute(store),
        Some(RequestData::Hmdel(param)) => param.execute(store),
        Some(RequestData::Hmexist(param)) => param.execute(store),
        Some(RequestData::Hgetall(param)) => param.execute(store),
        None => KvError::InvalidCommand("Request has no data".into()).into(),
        // 处理不了的返回一个啥都不包括的 Response，这样后续可以用 dispatch_stream 处理
        _ => CommandResponse::default(),
    }
}

/// 从 Request 中得到 Response，目前处理所有 PUBLISH/SUBSCRIBE/UNSUBSCRIBE
pub fn dispatch_stream(cmd: CommandRequest, topic: impl Topic) -> StreamingResponse {
    match cmd.request_data {
        Some(RequestData::Subscribe(param)) => param.execute(topic),
        Some(RequestData::Unsubscribe(param)) => param.execute(topic),
        Some(RequestData::Publish(param)) => param.execute(topic),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use http::StatusCode;
    use tokio_stream::StreamExt;
    use tracing::info;

    use super::*;
    use crate::{MemTable, Value};

    #[tokio::test]
    async fn service_should_work() {
        // service结构应至少包含Storage
        let service: Service = ServiceInner::new(MemTable::new()).into();

        // service 可以运行在多线程环境下，它的 clone 应该是轻量级的
        let cloned = service.clone();

        // 创建一个线程，在 table 中写入 key, value
        tokio::spawn(async move {
            let mut res = cloned.execute(CommandRequest::new_hset("table", "key", "value"));
            let data = res.next().await.unwrap();
            assert_res_ok(&data, &[Value::default()], &[]);
        })
        .await
        .unwrap();

        // 在当前线程下读取 table 的 key 返回 value
        let mut res = service.execute(CommandRequest::new_hget("table", "key"));
        let data = res.next().await.unwrap();
        assert_res_ok(&data, &["value".into()], &[]);
    }

    #[tokio::test]
    async fn event_registration_should_work() {
        fn b(cmd: &CommandRequest) {
            info!("Got {:?}", cmd);
        }

        fn c(res: &CommandResponse) {
            info!("Got {:?}", res);
        }

        fn d(res: &mut CommandResponse) {
            res.status = StatusCode::CREATED.as_u16() as _;
        }

        fn e() {
            info!("Data is sent");
        }

        let service: Service = ServiceInner::new(MemTable::default())
            .fn_received(|_cmd: &CommandRequest| {})
            .fn_received(b)
            .fn_executed(c)
            .fn_before_send(d)
            .fn_after_send(e)
            .into();

        let mut res = service.execute(CommandRequest::new_hset("table", "key", "value"));
        let data = res.next().await.unwrap();
        assert_eq!(data.status, StatusCode::CREATED.as_u16() as _);
        assert_eq!(data.message, "");
        assert_eq!(data.values, vec![Value::default()]);
    }
}

#[cfg(test)]
use crate::{Kvpair, Value};

// 测试成功的返回结果
#[cfg(test)]
pub fn assert_res_ok(res: &CommandResponse, values: &[Value], pairs: &[Kvpair]) {
    let mut sorted_pairs = res.pairs.clone();
    sorted_pairs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(res.status, 200);
    assert_eq!(res.message, "");
    assert_eq!(res.values, values);
    assert_eq!(sorted_pairs, pairs);
}

// 测试失败的返回结果
#[cfg(test)]
pub fn assert_res_error(res: &CommandResponse, code: u32, msg: &str) {
    assert_eq!(res.status, code);
    assert!(res.message.contains(msg));
    assert_eq!(res.values, &[]);
    assert_eq!(res.pairs, &[]);
}
