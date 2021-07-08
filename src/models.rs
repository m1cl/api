use anyhow::{Error, Result};
use argonautica::{Hasher, Verifier};
use async_graphql::SimpleObject;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::env;

#[derive(SimpleObject, FromRow, Deserialize, Serialize)]
pub struct User {
    pub id: sqlx::types::Uuid,
    username: String,
    password: String,
    created_at: DateTime<Utc>,
}
fn validate_passwords(password: &str, db_password: &str) -> Result<bool, argonautica::Error> {
    let secret_key = env::var("SECRET_KEY").expect("No secret key in .env file");
    let mut verifier = Verifier::default();
    verifier
        .with_secret_key(secret_key)
        .with_password(password)
        .with_hash(db_password)
        .verify()
}

impl User {
    pub async fn create(pool: &PgPool, username: &str, password: &str) -> Result<User> {
        let secret_key = env::var("SECRET_KEY").expect("No secret key in .env file");
        let mut hasher = Hasher::default();

        // This is blocking.. so I could take a while
        let hash = hasher
            .with_password(password)
            .with_secret_key(secret_key)
            .hash()
            .unwrap();
        //
        let row = sqlx::query!(
            "INSERT INTO users(username, password) Values($1, $2) returning id, created_at",
            username,
            hash
        )
        .fetch_one(pool)
        .await?;

        Ok(User {
            id: row.id,
            username: username.to_string(),
            password: password.to_string(),
            created_at: row.created_at,
        })
    }
    pub async fn get_all_users(pool: &PgPool) -> Result<Vec<User>> {
        let row = sqlx::query_as!(User, "Select * from users")
            .fetch_all(pool)
            .await?;
        Ok(row)
    }
    pub async fn get_user_by_name(pool: &PgPool, username: &str) -> Result<User> {
        let row = sqlx::query_as!(User, "Select * from users where username = $1", username)
            .fetch_one(pool)
            .await?;
        Ok(row)
    }
    // TODO: return Result<Token>
    pub async fn login(pool: &PgPool, username: &str, password: &str) -> Result<User> {
        let row = sqlx::query_as!(User, "Select * from users where username = $1", username)
            .fetch_one(pool)
            .await
            .unwrap();

        match validate_passwords(password, &row.password) {
            Ok(_) => Ok(row),
            Err(err) => {
                eprint!("{:?}", err);
                Err(anyhow::Error::msg("Nutzername und/oder Passwort falsch"))
            }
        }
    }
}
