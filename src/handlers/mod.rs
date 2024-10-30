//list
//add
//delete
//update

use std::convert::Infallible;
use log::{debug, info};
use warp::http::StatusCode;
use crate::models::{Db, QueryOptions, Todo};

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

pub async fn add_todo(create: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
    debug!("add_todo: {:?}", create);
    let mut todos = db.lock().await;
    for todo in todos.iter() {
        if todo.id == create.id {
            debug!("    -> id already exists: {}", create.id);
            return Ok(StatusCode::BAD_REQUEST);
        }
    }
    todos.push(create);
    Ok(StatusCode::OK)
}

pub async fn del_todo(del: usize, db: Db) -> Result<impl warp::Reply, Infallible> {
    debug!("del_todo: id={}", del);
    let mut todos = db.lock().await;
    let len = todos.len();
    todos.retain(|todo| todo.id != del);
    if todos.len() < len {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

pub async fn update_todo(id: usize, update: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
    debug!("update_todo: id={}, todo={:?}", id, update);
    let mut todos = db.lock().await;
    for todo in todos.iter_mut() {
        if todo.id == id {
            *todo = update;
            return Ok(StatusCode::OK);
        }
    }
    debug!("    -> todo id not found!");
    Ok(StatusCode::NOT_FOUND)
}