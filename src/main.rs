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
