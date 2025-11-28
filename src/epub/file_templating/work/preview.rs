use askama::Template;
use crate::html::types::Work;

#[derive(Template)]
#[template(path = "work/preview.html")]
pub struct WorkPreview <'a> {
    pub work: &'a Work,
}