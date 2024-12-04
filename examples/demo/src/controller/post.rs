use crate::{
    app::{Demo, DemoContext},
    model::Post,
    view,
};
use axum::{extract::Form, response::IntoResponse};
use lowboy::{
    error::LowboyError,
    extract::{DatabaseConnection, EnsureAppUser},
    model::Model as _,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PostCreateForm {
    message: String,
}

pub async fn create(
    EnsureAppUser(author): EnsureAppUser<Demo, DemoContext>,
    DatabaseConnection(mut conn): DatabaseConnection,
    Form(input): Form<PostCreateForm>,
) -> Result<impl IntoResponse, LowboyError> {
    let new_post = Post::create_record(author.id, &input.message)
        .save(&mut conn)
        .await?;
    let mut post = Post::load(new_post.id, &mut conn).await?;

    post.content = "New post content!".to_string();
    post.update_record().save(&mut conn).await?;

    let form = view::PostForm {};
    let post = view::Post { post };

    Ok(format!("{form}{post}"))
}
