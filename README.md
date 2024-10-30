# A Sample Rust Http Server Project

This is a sample project to demonstrate how to create a simple http server in Rust.

## Quick Start

we can create a todo project to demonstrate how to create a simple http server in Rust.

feature:

- list_todo(`GET` /todos): list all todos
- create_todo(`POST` /todos): create a new todo
- update_todo(`PATCH` /todos/{id}): update a todo
- delete_todo(`DELETE` /todos/{id}): delete a todo

### Dependencies

```bash
# log and pretty_env_logger for logging
cargo add log
cargo add pretty_env_logger

# warp and tokio for http server
cargo add warp
cargo add tokio --features full

# serde for serialization
cargo add serde --features derive
``` 

### project structure

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

- `src/main.rs`: The main entry point of the application.
- `src/filters`: Contains the filters for the application.
- `src/handlers`: Contains the handlers for the application.
- `src/models`: Contains the models for the application.

### create models

```bash
mkdir -p src/models
touch src/models/mod.rs
```

create todo model in `src/models/mod.rs`:

todo models has three fields: `id`, `title`, `completed`.

```rust
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}
```

add `serde` derive to `Todo` struct:
add Deserialize can parse json to Todo struct.
add Serialize can convert Todo struct to json.
add Debug can use {:?} to print Todo struct.

```rust
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}
```

create query todos conditions in `src/models/mod.rs`:
Option is can return None or Some(value).

```rust
#[derive(Debug, Deserialize)]
pub struct QueryOptions {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub complete: Option<bool>,
}
```

we use Vec store todos. because we use mulit-thread, so we use Arc<Mutex<Vec<Todo>>> to store todos.

Arc: Arc is a thread-safe reference-counting pointer.
Mutex: Mutex is a mutual exclusion primitive useful for protecting shared data.

```rust
pub fn init_db() -> Arc<Mutex<Vec<Todo>>> {
    Arc::new(Mutex::new(Vec::new()))
}
``` 

but `Arc<Mutex<Vec<Todo>>>` type is so long, we can use type alias to simplify it.

```rust
pub type Db = Arc<Mutex<Vec<Todo>>>;

pub fn init_db() -> Db {
    Arc::new(Mutex::new(Vec::new()))
}
``` 

### create handlers

first we create a `handlers/mod.rs` file:

```bash
mkdir -p src/handlers
touch src/handlers/mod.rs
```

create a function to list all todos in `src/handlers/mod.rs`:

because we use warp to create http server, so we need to return a `impl warp::Reply` type.

1. we use `info!` to print log.
2. we use `db.lock().await` to get todos.
3. we use `filter` to filter todos by `opts.complete`, `opts.offset`, `opts.limit`.
4. we use `warp::reply::json` to return todos.

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

other function will no further details...
jump to [handlers/mod.rs](src/handlers/mod.rs)

### create filters

filter is a matcher http request, and call handler to process request.

mkdir filters and create mod.rs file.

```bash
mkdir -p src/filters
touch src/filters/mod.rs
```

create a function to pass db to handler in `src/filters/mod.rs`:

```rust
fn with_db(db: Db) -> impl Filter<Extract=(Db,), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}
```

we will create a function to list todos in `src/filters/mod.rs`:
it can match `GET /todos` request, and call `handlers::list_todos` to process request.

error=warp:: rejection Specifies the error type returned by this filter on a failed or rejected request.
path!("todos") match /todos request.
get() match GET method.
query::<QueryOptions>() parse query string to QueryOptions struct.
with_db(db) pass db to handler.
and_then(handlers::list_todos) call handlers::list_todos to process request.

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

### create main.rs

**import custom carte module.**

- `models`: import `Db` type and `init_db` function.
- `filters`: import `todos` function.
- `handlers`: import `list_todos` function.

```rust
mod models;
mod filters;
mod handlers;
```

**import Filter**
`use warp::Filter;` to import Filter trait, because we use Filter in main function.

> if not import Filter will not find,because Filter is not in available range.
> `use warp::Filter;`

**import #[tokio::main]**
add `#[tokio::main]` to main function, because we use async function in main function.

init_db to create a db instance.

```rust
let db: Db = models::init_db();
```

init api to create todos api.

```rust
let api = filters::todos(db);
```

init route to add log to todos api.

```rust
let route = api.with(warp::log("todos"));
```

`warp::serve(route).run(([127, 0, 0, 1], 3030)).await;` to start http server.

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
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "coding_event::handlers=debug,todos=info");
    }

    pretty_env_logger::init();
    let db: Db = models::init_db();
    let api = filters::todos(db);
    let route = api.with(warp::log("todos"));
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}
```

