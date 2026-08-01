# SoulMsg

[EN](./README.md)

基于 protobuf 的 schema 校验消息层：payload 用 [protobuf](https://protobuf.dev/) 编码，外层包裹携带密码学哈希（Blake3）的信封，用于**消息身份**与**定义版本**校验。

## 概述

SoulMsg 在 protobuf payload 之上增加哈希信封：

```
[name_hash:32][version_hash:32][payload_len:u32(LE)][protobuf payload]
```

- `name_hash` —— 按短消息名标识类型（schema 编辑时保持稳定）。
- `version_hash` —— 覆盖完整定义（消息名 + 字段），任何 schema 变化都会检测到。
- 两个哈希均从 protoc **file descriptor set** 规范计算，因此只要对同一 `.proto` 哈希，各语言产出**逐字节一致**（已在 Rust / Python / Go / Kotlin 验证，见 `proto_poc/verification`）。

解码为策略驱动：`Strict`（默认）拒绝哈希不匹配；`Lenient` 跳过哈希校验，依赖 protobuf 的容忍式 schema 演进。

## 特性

- **protobuf payload**：复用整个 protobuf 生态（工具链、代码生成、conformance）
- **schema 身份**：线上携带密码学 `name_hash` / `version_hash`
- **默认严格、可按需宽松**：严格校验或容忍演进
- **跨语言**：Python / Go / Kotlin 均有薄 `soulmsg` 绑定，字节一致
- **仅字节层**：库只产出/消费字节；传输（zenoh 等）自行携带

## 安装

```toml
[dependencies]
soul_msg = "0.2"
prost = "0.14"

[build-dependencies]
prost-build = "0.14"
```

## 用法（Rust）

### 1. 定义消息并生成代码

编写 `.proto`，用 prost-build 编译，并给模块挂上 `#[smsg]` 宏。宏在编译期读取 descriptor set，为匹配包内的每个消息生成 `EnvelopeMeta`：

```rust
#[smsg("descriptors.pb")] // 由 prost-build / protoc --descriptor_set_out 产出
pub mod chat {
    include!(concat!(env!("OUT_DIR"), "/chat.rs"));
}
```

### 2. 序列化 / 反序列化

```rust
use soul_msg::{Envelope, EnvelopeError, Policy};

// 发送：类型即 schema，无需运行时元数据。
let msg = chat::ChatMessage {
    sender: "Alice".to_string(),
    content: "Hello, World!".to_string(),
    timestamp: 1_699_999_999,
};
let wire: Vec<u8> = Envelope::new_typed(&msg).to_bytes();

// 接收：默认严格。
let received: chat::ChatMessage = Envelope::try_deserialize_typed(&wire)?;

// 接收：宽松（protobuf 容忍演进）。
let received: chat::ChatMessage =
    Envelope::try_deserialize_with_policy_typed(&wire, Policy::Lenient)?;
```

### 3. 运行时 Schema（动态 / 分发）

当收包时不知道具体类型（如订阅者接收多种消息），可在运行时加载 descriptor set 并 peek 哈希分发：

```rust
use soul_msg::Schema;

let schema = Schema::from_descriptor_set(include_bytes!("descriptors.pb"))?;
let (name_hash, version_hash) = Envelope::peek(&wire)?;
let meta = schema.by_name_hash(&name_hash); // -> MessageRef
```

## 跨语言绑定

各语言使用自己的 protobuf 运行时；`soulmsg` 绑定只增加约 100 行哈希 + 分帧 + 策略代码，均与 Rust 逐字节一致验证：

| 语言 | 绑定 | payload 编解码 |
|------|------|----------------|
| Rust | `soul_msg` + `smsg_macro` | prost |
| Python | `bindings/python/soulmsg` | protobuf（`pip install protobuf blake3`） |
| Go | `bindings/go/soulmsg` | protobuf-go + blake3 |
| Kotlin/JVM | `bindings/kotlin/soulmsg` | protobuf-java + BouncyCastle |

逐字节一致性验证见 `proto_poc/verification/README.md`。

## 错误处理

- `NotAnEnvelope` —— 数据过短 / payload 长度分帧不符
- `TypeMismatch` —— `name_hash` 与期望消息不符
- `VersionMismatch` —— `version_hash` 与期望定义不符
- `DeserializeError` —— protobuf payload 解码失败

## License

MIT
