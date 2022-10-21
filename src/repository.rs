use crate::schema::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use crate::error::ApiError;
use actix_web::web;
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

    pub async fn crate_post(
        &self,
        new_post: NewPost,
    ) -> Result<Post, ApiError> {
        let mut conn = self.pool.get()?;
        let post = web::block(move || {
            diesel::insert_into(posts::table)
                .values(new_post)
                .get_result(&mut conn)
        })
        .await??;

        Ok(post)
    }

    pub async fn list_posts (
        &self,
    ) -> Result<Vec<Post>, ApiError> {
        let mut conn = self.pool.get()?;
        let res = web::block(move || {
            posts::table.load(&mut conn)
        })
        .await??;
        
        Ok(res)
    }

    pub async fn get_post(&self, id: i32) -> Result<Post, ApiError> {
        let mut conn = self.pool.get()?;
        let res = web::block(move || {
            posts::table.find(id)
                .first(&mut conn)
                .optional()
        })
        .await??
        .ok_or(ApiError::NotFound)?;

        Ok(res)
    }
}

#[derive(Deserialize, diesel::Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    title: String,
    body: String,
}

#[derive(Serialize, diesel::Queryable)]
pub struct Post {
    id: i32,
    title: String,
    body: String,
    published: bool
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

