use askama::Template;
use crate::html::types::{Category, Work};

#[derive(Template)]
#[template(path = "toc.html")]
pub struct TableOfContents <'a> {
    pub output_name: String,
    pub categories: &'a [Category],
    pub works: &'a Vec<Work>
}