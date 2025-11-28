use std::collections::HashMap;

use askama::Template;
use crate::{epub::file_templating::category_index::CategoryListing, html::types::{Anchor, Category, Work}};

#[derive(Template)]
#[template(path = "work/introduction.html")]
pub struct WorkIntroduction <'a> {
    pub epub_fandoms_links: Vec<Anchor>,
    pub epub_relationships_links: Vec<Anchor>,
    pub epub_characters_links: Vec<Anchor>,
    pub epub_tags_links: Vec<Anchor>,
    pub work: &'a Work,
}

impl <'a> WorkIntroduction <'a> {
    pub(crate) fn new(work: &&'a Work, category_listings: &'a HashMap<Category, HashMap<String, CategoryListing>>) -> Self {

        let epub_link_from_category = | category: Category | move | fandom: &Anchor | {
            let listing_map = category_listings.get(&category).unwrap();
            let listing_id = listing_map.get(&fandom.link).unwrap().id;
            let epub_link = format!("../../indexes/{category}/{category}-{listing_id}-listing.xhtml");
            let link_name = fandom.name.clone();
            return Anchor {
                link: epub_link,
                name: link_name
            }
        };

        Self {
            epub_fandoms_links:       work.fandoms       .iter().map(epub_link_from_category(Category::Fandoms)).collect(), 
            epub_relationships_links: work.relationships .iter().map(epub_link_from_category(Category::Relationships)).collect(), 
            epub_characters_links:    work.characters    .iter().map(epub_link_from_category(Category::Characters)).collect(), 
            epub_tags_links:          work.tags          .iter().map(epub_link_from_category(Category::Tags)).collect(), 
            work: work 
        }
    }
}