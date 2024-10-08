# rsdb - a simple kv store

一个简单的键值存储系统。内部封装了rocksdb类库和一个简单的通信协议。整体采用rust开发、支持rust和python的客户端驱动。

## workspace 项目说明

- `packet` 一个简单的通信协议实现
- `storage` 封装了存储层（rocksdb）的基本操作，定义了一个多数据库句柄结构和一些常量
- `server` 服务器实现，建立在packet和storage之上，支持 tcp/ip socket 和 unix domain socket
- `client` 客户端工具
- `rsdbrs` rust语言驱动
- `rsdbpy` python语言驱动
- `benchmarks` 性能测试工具
