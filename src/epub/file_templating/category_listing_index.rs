use askama::Template;
use crate::html::types::*;

#[derive(Template)]
#[template(path = "category-listing-index.html")]
pub struct CategoryListingIndex <'a> {
    pub category: String,
    pub listing_name: &'a String,
    pub listing: &'a Vec<&'a WorkStruct>
}