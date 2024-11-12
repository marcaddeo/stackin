use crate::model;
use askama::Template;

#[derive(Clone, Template)]
#[template(path = "components/post.html")]
pub struct Post {
    pub post: model::Post,
}