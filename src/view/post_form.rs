use askama::Template;

#[derive(Default, Template)]
#[template(path = "components/post-form.html")]
pub struct PostForm {
    pub swap_oob: bool,
}
