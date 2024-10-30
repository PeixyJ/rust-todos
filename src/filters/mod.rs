use warp::Filter;
use crate::handlers;
use crate::models::{Db, QueryOptions};

pub fn todos(
    db: Db,
) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    todos_list(db.clone())
        .or(add_todo(db.clone()))
        .or(del_todo(db.clone()))
        .or(update_todo(db).clone())
}

pub fn todos_list(
    db: Db,
) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    warp::path!("todos")
        .and(warp::get())
        .and(warp::query::<QueryOptions>())
        .and(with_db(db))
        .and_then(handlers::list_todos)
}

pub fn add_todo(db: Db) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    warp::path!("todos")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(handlers::add_todo)
}

pub fn del_todo(db: Db) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    warp::path!("todos" / usize)
        .and(warp::delete())
        .and(with_db(db))
        .and_then(handlers::del_todo)
}

pub fn update_todo(db: Db) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
    warp::path!("todos" / usize)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(handlers::update_todo)
}

fn with_db(db: Db) -> impl Filter<Extract=(Db,), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}