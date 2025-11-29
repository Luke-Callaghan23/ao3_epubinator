use core::fmt;
use std::collections::HashMap;
use derivative::Derivative;

pub type HTMLString = String;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Work {
    pub id: usize,
    pub playback_id: usize,
    pub title: String,
    pub link: String,
    pub category_data: HashMap<Category, Vec<Anchor>>,
    pub series: Option<Series>,
    pub wc: String,             // string because AO3 gives us the word count with commas, and that is convenient
    #[derivative(Debug(format_with = "html_formatter"))]
    pub summary: HTMLString,
    pub author: Author,
    pub chapters: Vec<Chapter>,
}


#[derive(Debug, Clone)]
pub struct Anchor {
    pub name: String,
    pub link: String,
}

pub type Author = Anchor;

#[derive(Debug)]
#[allow(unused)]
pub struct Series {
    pub name: String,
    pub link: String,
    pub part_number: usize,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Chapter {
    pub order: usize,
    pub playback_id: usize,
    pub title: String,
    #[derivative(Debug(format_with = "html_formatter"))]
    pub summary: HTMLString,
    #[derivative(Debug(format_with = "html_formatter"))]
    pub data: HTMLString,
}



fn html_formatter(val: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", val.chars().take(10).collect::<String>())
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Category {
    Titles,
    Ratings,
    Categories,
    Fandoms,
    Relationships,
    Characters,
    Tags,
    Authors,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Category::Titles => "titles",
            Category::Ratings => "ratings",
            Category::Categories => "categories",
            Category::Fandoms => "fandoms",
            Category::Relationships => "relationships",
            Category::Characters => "characters",
            Category::Tags => "tags",
            Category::Authors => "authors",
        })
    }
}
