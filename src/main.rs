#[allow(dead_code)]
use crate::{post::Post, user::User};
use askama::Template;
use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use axum_extra::{headers, TypedHeader};
use flume::Receiver;
use futures::{Stream, StreamExt};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{convert::Infallible, time::Duration};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt as _};

mod id;
mod post;
mod user;

#[derive(Clone)]
struct App {
    pub database: SqlitePool,
    pub sse_event_rx: Receiver<Event>,
}

impl App {
    pub async fn new(sse_event_rx: Receiver<Event>) -> Self {
        let database = &format!(
            "sqlite://{}/target/database.sqlite3",
            std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        );
        let database = SqlitePoolOptions::new()
            .max_connections(3)
            .connect(database)
            .await
            .unwrap();

        let query = r"
        CREATE TABLE IF NOT EXISTS user (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            email TEXT NOT NULL,
            byline TEXT NOT NULL,
            avatar TEXT NOT NULL,
            UNIQUE(email)
        );

        CREATE TABLE IF NOT EXISTS post (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            author_id INTEGER NOT NULL,
            content TEXT NOT NULL
        );
        ";
        sqlx::query(query).execute(&database).await.unwrap();

        Self {
            database,
            sse_event_rx,
        }
    }
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomeTemplate {
    user: User,
    posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "components/post.html")]
struct PostTemplate<'p> {
    post: &'p Post,
}

async fn index(State(App { database, .. }): State<App>) -> HomeTemplate {
    let user = User::find_by_id(1, &database)
        .await
        .expect("uid 1 should exist");
    let posts = Post::find(5, &database).await.unwrap();

    HomeTemplate { user, posts }
}

async fn events(
    State(App { sse_event_rx, .. }): State<App>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("`{}` connected", user_agent.as_str());

    let stream = sse_event_rx.into_stream().map(Ok);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (tx, rx) = flume::bounded::<Event>(32);

    let app = App::new(rx).await;

    let scheduler = tokio_cron_scheduler::JobScheduler::new()
        .await
        .expect("job scheduler should be created");
    scheduler.start().await.expect("scheduler should start");

    let ctx = app.clone();
    scheduler
        .add(
            tokio_cron_scheduler::Job::new_async("every 30 seconds", move |_, _| {
                let ctx = ctx.clone();
                let tx = tx.clone();
                Box::pin(async move {
                    let mut post = Post::fake();
                    let user = User::insert(&post.author, &ctx.database)
                        .await
                        .expect("inserting user should work");
                    post.author = user;
                    let post = Post::insert(post, &ctx.database)
                        .await
                        .expect("inserting post should work");

                    tx.send(
                        Event::default()
                            .event("NewPost")
                            .data(PostTemplate { post: &post }.render().unwrap()),
                    )
                    .unwrap();

                    info!(
                        "Added new post by: {} {}",
                        post.author.first_name, post.author.last_name
                    );
                })
            })
            .expect("job creation should succeed"),
        )
        .await
        .expect("scheduler should allow adding job");

    // build our application with a route
    let app = Router::new()
        .nest_service("/dist", ServeDir::new("dist"))
        .route("/", get(index))
        .route("/events", get(events))
        .with_state(app);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
