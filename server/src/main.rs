use std::env;

use async_graphql::http::GraphiQLSource;
use axum::{
    response::{self, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use clap::{command, Args, Parser, Subcommand};
use dotenv::dotenv;
use my_hood_server::{
    config::Config,
    graphql::{get_schema, graphql_handler},
    token::login_handler,
    user::model::User,
    DB,
};
use tokio::net::TcpListener;

mod utils;

#[derive(Parser, Debug)]
#[command(name = "MyHood", version = "1.0", about = "An example CLI app", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the application
    Run,
    /// Create a super user
    CreateSuperUser(CreateSuperUserArgs),
}

#[derive(Args, Debug)]
struct CreateSuperUserArgs {
    email: String,
    password: String,
}

async fn run() -> anyhow::Result<()> {
    let db_url = env::var("DATABASE_URL").unwrap();
    let db = DB::connect(&db_url).await.unwrap();

    let host = env::var("HOST").unwrap();
    let port = env::var("PORT").unwrap();

    let config = Config::new();

    let schema = get_schema(db.clone(), config.clone());

    async fn graphql_playground() -> impl IntoResponse {
        response::Html(GraphiQLSource::build().endpoint("/").finish())
    }

    let app = Router::new()
        .route(
            "/",
            get(graphql_playground)
                // get(graphql_playground).post_service(GraphQL::new(schema)),
                .post(post(graphql_handler)),
        )
        .route("/auth", post(login_handler))
        .layer(Extension(schema))
        .layer(Extension(db));
    println!("Serving on http://{host}:{port}");
    axum::serve(
        TcpListener::bind(format!("{host}:{port}")).await.unwrap(),
        app,
    )
    .await
    .expect("Error spawning server");
    Ok(())
}

async fn create_super_user(email: String, password: String) -> anyhow::Result<User> {
    let password_hash = bcrypt::hash(password, 12)?;
    let db_url = env::var("DATABASE_URL").unwrap();
    let db = DB::connect(&db_url).await.unwrap();
    let mut tx = db.begin().await?;
    let user = sqlx::query_as!(
        User,
        r#"INSERT INTO "User" (password_hash, name, birthday, address, email)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *"#,
        password_hash,
        "SuperUser",
        "2021-01-01".parse::<chrono::NaiveDate>()?,
        "SuperUser's address",
        email,
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"INSERT INTO "GlobalAdmin" (user_id) VALUES ($1) RETURNING *"#,
        user.id
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(user)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Run => run().await,
        // Commands::CreateSuperUser => create_super_user().await?,
        Commands::CreateSuperUser(args) => {
            let user = create_super_user(args.email, args.password).await?;
            println!("Super user created: {:?}", user);
            Ok(())
        }
    }
}
