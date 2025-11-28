use askama::Template;
use crate::html::types::Category;

#[derive(Template)]
#[template(path = "index_index.html")]
pub struct IndexIndex <'a> {
    pub output_name: String,
    pub categories: &'a [Category],
}