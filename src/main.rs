extern crate dotenv;
use actix_web::{
    get, guard, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder,
};

use anyhow::Result;
use argonautica::Hasher;
use async_graphql::{Context, EmptySubscription, FieldResult, Object, Schema};
use async_graphql_actix_web::{Request, Response};
use env_logger::{Builder, Target};
use std::sync::Mutex;

use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};

mod models;
use models::User;
use std::env;

struct AppStateWithCounter {
    counter: Mutex<i32>,
}

// async fn index(schema: web::Data<Users>, req: Request) -> Response {
//     // schema.execute(req.into_inner()).await.into()
// }
#[tokio::main]
pub async fn establish_db_connection() -> Pool<Postgres> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set (.env file)");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap()
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
#[get("/all_posts")]
async fn get_all_posts() -> impl Responder {
    HttpResponse::Ok().body("All posts")
}

#[post("/login")]
async fn login(req_body: String) -> impl Responder {
    print!("\nRequest body:\t {:?}\n", req_body);
    HttpResponse::Ok().body("Hey there!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    print!("\nRequest body:\t {:?}\n", req_body);
    HttpResponse::Ok().body("Hey there!")
}

/// register graphql server with sqlx
async fn index(schema: web::Data<ServiceSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
}
async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct Query;

#[Object(extends)]
impl Query {
    async fn users<'a>(&self, ctx: &'a Context<'_>) -> FieldResult<Vec<User>> {
        let pool = ctx.data::<PgPool>().unwrap();
        let rows = User::get_all_users(pool).await?;
        Ok(rows)
    }
    async fn login<'a>(
        &self,
        ctx: &'a Context<'_>,
        username: String,
        password: String,
    ) -> FieldResult<User> {
        println!("attempt to login {} with {}", &username, &password);
        let pool = ctx.data::<PgPool>().unwrap();
        let row = User::login(pool, &username, &password).await?;
        Ok(row)
    }
    #[graphql(entity)]
    async fn user<'a>(&self, ctx: &'a Context<'_>, username: String) -> FieldResult<User> {
        let pool = ctx.data::<PgPool>().unwrap();
        let row = User::get_user_by_name(pool, &username).await?;
        Ok(row)
    }
}

struct Mutation;
#[async_graphql::Object]
impl Mutation {
    async fn create_users(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
    ) -> FieldResult<User> {
        let pool = ctx.data::<PgPool>().unwrap();
        let row = User::create(&pool, &username, &password).await?;
        Ok(row)
    }
}

type ServiceSchema = Schema<Query, Mutation, EmptySubscription>;

#[actix_web::main]
async fn main() -> Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });
    dotenv().ok();

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    // let host = env::var("HOST").expect("HOST is not set");
    // let port = env::var("PORT").expect("PORT is not set");
    let db_pool = PgPool::connect(&database_url).await?;

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(db_pool)
        .finish();

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(hello)
            .service(echo)
            .service(web::resource("/").guard(guard::Post()).to(index))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("localhost:8080")?
    .run()
    .await?;
    Ok(())
}
