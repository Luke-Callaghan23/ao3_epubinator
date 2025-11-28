use askama::Template;
use crate::html::types::{Anchor, Chapter};

#[derive(Template)]
#[template(path = "work/chapter.html")]
pub struct WorkChapter <'a> {
    pub work_title: &'a String,
    pub work_author: &'a Anchor,
    pub chapter: &'a Chapter
}