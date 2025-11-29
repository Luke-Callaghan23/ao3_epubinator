use std::collections::HashMap;

use askama::Template;
use crate::{epub::file_templating::category_index::CategoryListing, html::types::{Anchor, Category, Work}};

#[derive(Template)]
#[template(path = "work/introduction.html")]
pub struct WorkIntroduction <'a> {
    pub epub_ratings_links: Vec<Anchor>,
    pub epub_categories_links: Vec<Anchor>,
    pub epub_fandoms_links: Vec<Anchor>,
    pub epub_relationships_links: Vec<Anchor>,
    pub epub_characters_links: Vec<Anchor>,
    pub epub_tags_links: Vec<Anchor>,
    pub work: &'a Work,
}

impl <'a> WorkIntroduction <'a> {
    pub(crate) fn new(work: &&'a Work, category_listings: &'a HashMap<Category, HashMap<String, CategoryListing>>) -> Self {

        let epub_link_from_category = | work: &Work, category: Category | -> Vec<Anchor> {
            work.category_data.get(&category).unwrap().iter().map(| anchor | {
                let listing_map = category_listings.get(&category).unwrap();
                let listing_id = listing_map.get(&anchor.link).unwrap().id;
                let epub_link = format!("../../indexes/{category}/{category}-{listing_id}-listing.xhtml");
                let link_name = anchor.name.clone();
                return Anchor {
                    link: epub_link,
                    name: link_name
                }
            }).collect()
        };

        Self {
            epub_ratings_links:       epub_link_from_category(&work, Category::Ratings), 
            epub_categories_links:    epub_link_from_category(&work, Category::Categories), 
            epub_fandoms_links:       epub_link_from_category(&work, Category::Fandoms), 
            epub_relationships_links: epub_link_from_category(&work, Category::Relationships), 
            epub_characters_links:    epub_link_from_category(&work, Category::Characters), 
            epub_tags_links:          epub_link_from_category(&work, Category::Tags), 
            work: work 
        }
    }
}