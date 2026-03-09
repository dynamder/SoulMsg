# SoulMsg

[EN](./README.md)

SoulMsg是一个 Rust 消息序列化框架，提供类型安全的版本控制支持。

## 概述

SoulMsg 是一个将自定义 DSL（`.smsg` 文件）与自动 Rust 结构体生成相结合的消息序列化库。消息被封装在包含加密哈希（Blake3）的SmsgEnvelope中，用于版本和类型验证，支持分布式系统中的安全模式演进。

## 特性

- **自定义 DSL**：使用简单的 `.smsg` 文件格式定义消息
- **过程宏生成**：从 `.smsg` 定义自动生成 Rust 结构体
- **类型安全**：加密名称哈希防止将消息反序列化为错误的类型
- **版本跟踪**：版本哈希支持检测模式变更
- **Zenoh 集成**：基于 Zenoh 实现高效序列化/反序列化（目前仅支持 Zenoh，即将支持 Serde）

## 安装

添加到 `Cargo.toml`：

```toml
[dependencies]
soul_msg = "0.1"
zenoh = "1.7"
zenoh-ext = "1.7"
```

## 使用方法

### 定义消息（.smsg 文件）

创建 `.smsg` 文件来定义您的消息类型：

```smsg
message ChatMessage {
    string sender
    string content
    int64 timestamp
}

message Position {
    float64 x
    float64 y
    float64 z
}
```

### 生成 Rust 代码

使用 `#[smsg]` 属性宏从 `.smsg` 文件生成 Rust 代码：

```rust
use soul_msg::{smsg, MessageMeta, SmsgEnvelope};
use zenoh_ext::z_serialize;

#[smsg(category = file, path = "messages.smsg")]
pub mod chat_msgs {}
```

### 序列化和反序列化

```rust
use soul_msg::SmsgEnvelope;
use zenoh_ext::z_serialize;

// 创建消息
let msg = chat_msgs::ChatMessage {
    sender: "Alice".to_string(),
    content: "Hello, World!".to_string(),
    timestamp: 1699999999,
};

// 封装到带有版本/名称哈希的信封中
let envelope = SmsgEnvelope::new(msg);

// 序列化
let serialized = z_serialize(&envelope);
let bytes = serialized.to_bytes();

// 反序列化（带有类型和版本验证）
let received: chat_msgs::ChatMessage =
    SmsgEnvelope::try_deserialize(bytes).unwrap();
```

### 仅支持 Zenoh（Serde 即将支持）

目前，SoulMsg 仅支持通过 **Zenoh** 和 `zenoh-ext` 进行序列化。这是分布式系统和发布/订阅消息的默认推荐后端。

**Serde 支持正在规划中**，将在未来版本中添加，以满足不需要 Zenoh 的使用场景。

## 包支持

SoulMsg 支持将消息组织成**包**，适用于较大的项目。包是一个包含以下内容的目录：

1. 定义包元数据的 `package.toml` 文件
2. 在子目录中组织的多个 `.smsg` 文件

### 创建包

创建如下目录结构：

```
mypackage/
├── package.toml
├── person.smsg
└── orders/
    └── order.smsg
```

`package.toml` 应包含：

```toml
[package]
name = "mypackage"
version = "1.0.0"
edition = "2026"
```

像往常一样在 `.smsg` 文件中定义消息。子目录变成 Rust 模块。

### 使用包

使用 `category = package` 属性：

```rust
#[smsg(category = package, path = "path/to/mypackage")]
pub mod mypackage {}
```

这将生成与您的目录结构匹配的模块层次结构：

```rust
use mypackage::person::Person;
use mypackage::orders::Order;
```

包支持：
- **模块化组织**：将相关消息分组
- **命名空间**：避免消息类型之间的名称冲突
- **选择性导入**：仅导入需要的消息

## 支持的类型

| .smsg 类型 | Rust 类型 |
|------------|-----------|
| `string`   | `String`  |
| `int32`    | `i32`     |
| `int64`    | `i64`     |
| `float32`  | `f32`     |
| `float64`  | `f64`     |
| `bool`     | `bool`    |
| `bytes`    | `Vec<u8>` |

也支持嵌套消息。

## 错误处理

`SmsgEnvelope::try_deserialize` 对各种失败情况返回 `EnvelopeError`：

- `NotAnEnvelope`：数据太短或长度前缀无效
- `TypeMismatch`：消息名称哈希与预期类型不匹配
- `VersionMismatch`：消息版本哈希与预期版本不匹配
- `DeserializeError`：反序列化有效载荷失败

## 许可证

MIT

---

[English README](./README.md)
