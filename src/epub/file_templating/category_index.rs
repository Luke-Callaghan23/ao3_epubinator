use askama::Template;
use crate::html::types::*;

pub struct CategoryListing <'a> {
    pub id: usize,
    pub name: String,
    pub count: usize,
    pub works: Vec<&'a Work>,
}

#[derive(Template)]
#[template(path = "category-index.html")]
pub struct CategoryIndex <'a> {
    pub category: String,
    pub categories: Vec<&'a CategoryListing<'a>>
}