use askama::Template;
use crate::html::types::WorkStruct;

#[derive(Template)]
#[template(path = "work/preview.html")]
pub struct WorkPreview <'a> {
    pub work: &'a WorkStruct,
}