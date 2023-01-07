use sqlx::{Result, SqliteConnection};

struct User {
    id: i64,
    name: String,
}

async fn get_user_by_id(db: &mut SqliteConnection, id: i64) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT id, name FROM User WHERE id = ?",
        id,
    )
    .fetch_one(db)
    .await
}
