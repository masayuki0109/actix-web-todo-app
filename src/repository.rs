use crate::error::ApiError;
use crate::schema::*;
use actix_web::web;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub struct Repository {
    pool: DbPool,
}

impl Repository {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create a pool");
        Self { pool }
    }

    pub async fn crate_todo(&self, new_todo: NewTodo) -> Result<Todo, ApiError> {
        let mut conn = self.pool.get()?;
        let todo = web::block(move || {
            diesel::insert_into(todos::table)
                .values(new_todo)
                .get_result(&mut conn)
        })
        .await??;

        Ok(todo)
    }

    pub async fn list_todos(&self) -> Result<Vec<Todo>, ApiError> {
        let mut conn = self.pool.get()?;
        let res = web::block(move || todos::table.load(&mut conn)).await??;

        Ok(res)
    }

    pub async fn get_todo(&self, id: i32) -> Result<Todo, ApiError> {
        let mut conn = self.pool.get()?;
        let res = web::block(move || todos::table.find(id).first(&mut conn).optional())
            .await??
            .ok_or(ApiError::NotFound)?;

        Ok(res)
    }

    pub async fn update_todo(&self, id: i32, changeset: TodoChangeset) -> Result<Todo, ApiError> {
        let mut conn = self.pool.get()?;
        let todo = web::block(move || {
            diesel::update(todos::table.find(id)).set(changeset).get_result(&mut conn)
        })
        .await??;

        Ok(todo)
    }

    pub async fn done_todo(&self, id: i32, done: bool) -> Result<(), ApiError> {
        let mut conn = self.pool.get()?;
        web::block(move || {
            diesel::update(todos::table.find(id))
                .set(todos::done.eq(done))
                .execute(&mut conn)
        })
        .await??;

        Ok(())
    }

    pub async fn delete_todo(&self, id: i32) -> Result<(), ApiError> {
        let mut conn = self.pool.get()?;
        web::block(move || diesel::delete(todos::table.find(id)).execute(&mut conn)).await??;

        Ok(())
    }
}

#[derive(Deserialize, diesel::Insertable)]
#[diesel(table_name = todos)]
pub struct NewTodo {
    title: String,
    description: Option<String>,
}

#[derive(Serialize, diesel::Queryable)]
pub struct Todo {
    id: i32,
    title: String,
    description: Option<String>,
    done: bool,
    published: bool,
}

#[derive(Deserialize)]
pub struct TodoDoneRequest {
    pub done: bool,
}

#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = todos)]
pub struct TodoChangeset {
    title: Option<String>,
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conn() {
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let repo = Repository::new(&database_url);
        assert!(repo.pool.get().is_ok());
    }
}
