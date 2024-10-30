# 一个示例 Rust Http 服务器项目

这是一个示例项目，展示了如何在 Rust 中创建一个简单的 http 服务器。

## 快速开始

我们可以创建一个待办事项项目来展示如何在 Rust 中创建一个简单的 http 服务器。

特性：

- list_todo(`GET` /todos)：列出所有待办事项
- create_todo(`POST` /todos)：创建一个新的待办事项
- update_todo(`PATCH` /todos/{id})：更新一个待办事项
- delete_todo(`DELETE` /todos/{id})：删除一个待办事项

### 依赖

```bash
# log 和 pretty_env_logger 用于日志记录
cargo add log
cargo add pretty_env_logger

# warp 和 tokio 用于 http 服务器
cargo add warp
cargo add tokio --features full

# serde 用于序列化
cargo add serde --features derive
``` 

### 项目结构

```text
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src
│   ├── filters
│   │   └── mod.rs
│   ├── handlers
│   │   └── mod.rs
│   ├── main.rs
│   └── models
│       └── mod.rs
└── tests
```

- `src/main.rs`：应用程序的主入口点。
- `src/filters`：包含应用程序的过滤器。
- `src/handlers`：包含应用程序的处理程序。
- `src/models`：包含应用程序的模型。

### 创建模型

```bash
mkdir -p src/models
touch src/models/mod.rs
```

在 `src/models/mod.rs` 中创建待办事项模型：

待办事项模型有三个字段：`id`、`title`、`completed`。

```rust
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}
```

为 `Todo` 结构体添加 `serde` 派生：

添加 Deserialize 可以解析 json 到 Todo 结构体。
添加 Serialize 可以转换 Todo 结构体到 json。
添加 Debug 可以使用 {:?} 打印 Todo 结构体。

```rust
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}
```

在 `src/models/mod.rs` 中创建查询待办事项条件：

Option 是可以返回 None 或 Some(value)。

```rust
#[derive(Debug, Deserialize)]
pub struct QueryOptions {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub complete: Option<bool>,
}
```

我们使用 Vec 存储待办事项。因为我们使用多线程，所以我们使用 Arc<Mutex<Vec<Todo>>> 来存储待办事项。

Arc：Arc 是一个线程安全的引用计数指针。
Mutex：Mutex 是一个互斥原语，用于保护共享数据。

```rust
pub fn init_db() -> Arc<Mutex<Vec<Todo>>> {
    Arc::new(Mutex::new(Vec::new()))
}
``` 

但是 `Arc<Mutex<Vec<Todo>>>` 类型太长了，我们可以使用类型别名来简化它。

```rust
pub type Db = Arc<Mutex<Vec<Todo>>>;
pub fn init_db() -> Db {
    Arc::new(Mutex::new(Vec::new()))
}
``` 

### 创建处理程序

首先我们创建一个 `handlers/mod.rs` 文件：

```bash
mkdir -p src/handlers
touch src/handlers/mod.rs
```

在 `src/handlers/mod.rs` 中创建一个函数来列出所有待办事项：

因为我们使用 warp 来创建 http 服务器，所以我们需要返回一个 `impl warp::Reply` 类型。

1. 我们使用 `info!` 来打印日志。
2. 我们使用 `db.lock().await` 来获取待办事项。
3. 我们使用 `filter` 根据 `opts.complete`、`opts.offset`、`opts.limit` 过滤待办事项。
4. 我们使用 `warp::reply::json` 返回待办事项。

```rust
pub async fn list_todos(opts: QueryOptions, db: Db) -> Result<impl warp::Reply, Infallible> {
    info!("list_todos: {:?}", opts);
    let todos = db.lock().await;
    let todos: Vec<Todo> = todos.clone()
        .into_iter()
        .filter(|todo| todo.complete == opts.complete.unwrap_or(todo.complete))
        .skip(opts.offset.unwrap_or(0))
        .take(opts.limit.unwrap_or(std::usize::MAX))
        .collect();
    Ok(warp::reply::json(&todos))
}
```

其他函数将不再详细说明...
跳转到 [handlers/mod.rs](src/handlers/mod.rs)

### 创建过滤器

过滤器是匹配 http 请求，并调用处理程序来处理请求。

创建 filters 目录并创建 mod.rs 文件。

```bash
mkdir -p src/filters
touch src/filters/mod.rs
```

在 `src/filters/mod.rs` 中创建一个函数来传递数据库到处理程序：

```rust
fn with_db(db: Db) -> impl Filter<Extract=(Db,), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}
```

我们将在 `src/filters/mod.rs` 中创建一个函数来列出待办事项：
它可以匹配 `GET /todos` 请求，并调用 `handlers::list_todos` 来处理请求。

error=warp:: rejection 指定了这个过滤器在请求失败或被拒绝时返回的错误类型。
path!("todos") 匹配 /todos 请求。
get() 匹配 GET 方法。
query::<QueryOptions>() 解析查询字符串到 QueryOptions 结构体。
with_db(db) 传递数据库到处理程序。
and_then(handlers::list_todos) 调用 handlers::list_todos 来处理请求。

```rust
pub fn todos_list(
    db: Db,
) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    warp::path!("todos")
        .and(warp::get())
        .and(warp::query::<QueryOptions>())
        .and(with_db(db))
        .and_then(handlers::list_todos)
}
```

### 创建 main.rs

**导入自定义 crate 模块。**

- `models`：导入 `Db` 类型和 `init_db` 函数。
- `filters`：导入 `todos` 函数。
- `handlers`：导入 `list_todos` 函数。

```rust
mod models;
mod filters;
mod handlers;
```

**导入 Filter**
`use warp::Filter;` 导入 Filter trait，因为我们在 main 函数中使用 Filter。

> 如果没有导入 Filter 将找不到，因为 Filter 不在可用范围内。
> `use warp::Filter;`

**导入 #[tokio::main]**
给 main 函数添加 `#[tokio::main]`，因为我们在 main 函数中使用异步函数。

初始化数据库以创建一个数据库实例。

```rust
let db: Db = models::init_db();
```

初始化 api 以创建待办事项 api。

```rust
let api = filters::todos(db);
```

初始化路由以给待办事项 api 添加日志。

```rust
let route = api.with(warp::log("todos"));
```

`warp::serve(route).run(([127, 0, 0, 1], 3030)).await;` 启动 http 服务器。

```rust

#![deny(warnings)]
mod models;
mod filters;
mod handlers;

use std::env;
use warp::Filter;
use crate::models::Db;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // 设置 `RUST_LOG=todos=debug` 可以看到 debug 日志，
        // 这只显示访问日志。
        env::set_var("RUST_LOG", "coding_event::handlers=debug,todos=info");
    }

    pretty_env_logger::init();
    let db: Db = models::init_db();
    let api = filters::todos(db);
    let route = api.with(warp::log("todos"));
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}
```
