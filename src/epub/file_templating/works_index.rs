use askama::Template;
use crate::html::types::{Category, Work};

#[derive(Template)]
#[template(path = "works_index.html")]
pub struct WorksIndex <'a> {
    pub output_name: String,
    pub categories: &'a [Category],
    pub works: &'a Vec<Work>
}