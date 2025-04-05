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
    association::model::Association,
    config::Config,
    graphql::{get_schema, graphql_handler},
    oauth::{callback_handler, google_oauth_client},
    relations::model::{Relations, Role},
    token::login_handler,
    user::model::User,
    DB,
};
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use http::{Method, HeaderValue};

mod utils;

#[derive(Parser, Debug)]
#[command(name = "MyHood", version = "1.0", about = "An example CLI app", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the application.
    Run,
    /// Create a user.
    CreateUser(CreateUserArgs),
    /// Grant admin and treasurer permission to user in all associations.
    GrantAllPermissions(GrantAllPermissionsArgs),
}

#[derive(Args, Debug)]
struct CreateUserArgs {
    email: String,
    password: String,
}

#[derive(Args, Debug)]
struct GrantAllPermissionsArgs {
    user_id: uuid::Uuid,
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

    let allowed_origins = get_allowed_origins();

    // CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(graphql_playground).post(post(graphql_handler)))
        .route("/auth", post(login_handler))
        .route("/oauth/google/login", get(google_oauth_client))
        .route("/oauth/google/callback", get(callback_handler))
        .layer(Extension(schema))
        .layer(Extension(db))
        .layer(cors);

    println!("Serving on http://{host}:{port}");
    axum::serve(
        TcpListener::bind(format!("{host}:{port}")).await.unwrap(),
        app,
    )
    .await
    .expect("Error spawning server");
    Ok(())
}

fn get_allowed_origins() -> Vec<HeaderValue> {
    let origins = env::var("ALLOWED_ORIGINS").unwrap();
    
    // split string by comma, trim spaces and try to convert each item to HeaderValue
    origins
        .split(',')
        .map(|origin| origin.trim())
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect()
}

async fn create_user(email: String, password: String) -> anyhow::Result<User> {
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

    tx.commit().await?;
    Ok(user)
}

// Grant admin and treasurer permission to user in all associations
async fn grant_permissions(user_id: uuid::Uuid) -> anyhow::Result<()> {
    let db_url = env::var("DATABASE_URL").unwrap();
    let db = DB::connect(&db_url).await?;
    let associations = Association::read_all(&db).await?;

    let mut tx = db.begin().await?;

    for association in associations {
        Relations::create_association_role(
            &mut *tx,
            user_id,
            association.id,
            Role::Admin,
            false,
            None,
        )
        .await?;
        Relations::create_association_role(
            &mut *tx,
            user_id,
            association.id,
            Role::Member,
            false,
            None,
        )
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Run => run().await,
        Commands::CreateUser(args) => {
            let user = create_user(args.email, args.password).await?;
            println!("User created: {:?}", user);
            Ok(())
        }
        Commands::GrantAllPermissions(args) => {
            grant_permissions(args.user_id).await?;
            Ok(())
        }
    }
}
