# CCKVServer
这个项目的初衷是为了练习从零开始构建一个 KV 服务器。支持基本的[存储功能和发布订阅功能](#Commands)
# Architecture
![Overview of Server Architecture](./Architecture.png)
# Feature
- 存储：支持多种存储后端，包括基于内存的[DashMap](https://github.com/xacrimon/dashmap)，嵌入式的[rocksdb](https://github.com/rust-rocksdb/rust-rocksdb)、[sled](https://github.com/spacejam/sled)
- 加密协议：支持Tls协议或[Noise协议](https://noiseprotocol.org/noise.html)
- 多路复用：支持[Yamux协议](https://github.com/hashicorp/yamux/blob/master/spec.md)或[Quic协议](https://quicwg.org/)
- 监控和测量：由[opentelemetry](https://opentelemetry.io/)和[jaeger](https://www.jaegertracing.io/)集成
- 自定义的帧数据封装格式：每个数据帧的包头占四字节，包含长度、是否压缩及压缩格式等信息
# Usage
### 配置文件
服务端和客户端会读取由tool/gen_config.rs生成的配置文件
```plaintext
~/$ cargo run --bin gen_config -- -h
help generating client and server config

Usage: gen_config [OPTIONS] --protocol <PROTOCOL>

Options:
  -p, --protocol <PROTOCOL>          Tls files are required when using Quic and Tls.
                                     You may use tools/gen_cert.rs to generate these files. [possible values: tls, noise, quic]
  -a, --addr <ADDR>                  [default: 127.0.0.1:1973]
      --enable-jaeger                
      --enable-log-file              
      --log-level <LOG_LEVEL>        [default: info]
      --log-path <LOG_PATH>          [default: /tmp/kv-log]
      --log-rotation <LOG_ROTATION>  [default: daily] [possible values: hourly, daily, never]
      --storage <STORAGE>            [default: memtable]
  -h, --help                         Print help
```
### Running Server
```sh
cargo run --bin kvs
```
### Running Client
```sh
cargo run --bin kvc
```
## Commands
> Cli交互时会提供详细的命令使用提示
* Hget
* Hgetall
* Hmget
* Hset
* Hmset
* Hdel
* Hmdel
* Hexist
* Hmexist
* Publish
* Subscribe
* Unsubscribe

# Benchmark
### Pubsub
在100个订阅者的背景下，对Publish操作进行了基准测试。结果如下：
```plaintext
publishing              time:   [76.341 µs 76.824 µs 77.390 µs]
                        change: [-0.6231% +0.9935% +2.7754%] (p = 0.29 > 0.05)
```
每次发布操作的平均时间为76.341微秒。换算后，服务器每秒大约可处理13,097次发布操作
### 🚧More Benches...🚧