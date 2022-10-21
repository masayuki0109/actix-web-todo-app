mod error;
mod repository;
mod schema;

use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App, HttpResponse, HttpServer};
use error::ApiError;
use repository::{NewTodo, Repository};

use crate::repository::TodoDoneRequest;

#[actix_web::get("/todos")]
async fn list_todos(repo: web::Data<Repository>) -> Result<HttpResponse, ApiError> {
    let res = repo.list_todos().await?;
    Ok(HttpResponse::Ok().json(res))
}

#[actix_web::post("/todos")]
async fn create_todo(
    repo: web::Data<Repository>,
    new_todo: web::Json<NewTodo>,
) -> Result<HttpResponse, ApiError> {
    let new_todo = new_todo.into_inner();
    let post = repo.crate_todo(new_todo).await?;
    Ok(HttpResponse::Ok().json(post))
}

#[actix_web::get("/todos/{id}")]
async fn get_todo(
    repo: web::Data<Repository>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let res = repo.get_todo(id).await?;
    Ok(HttpResponse::Ok().json(res))
}

#[actix_web::put("/todos/{id}")]
async fn done_todo(
    repo: web::Data<Repository>,
    path: web::Path<i32>,
    req_done: web::Json<TodoDoneRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let req = req_done.into_inner();
    repo.done_todo(id, req.done).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[actix_web::delete("/todos/{id}")]
async fn delete_todo(
    repo: web::Data<Repository>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    repo.delete_todo(id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let repo = web::Data::new(Repository::new(&database_url));

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .service(create_todo)
            .service(list_todos)
            .service(get_todo)
            .service(done_todo)
            .service(delete_todo)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
