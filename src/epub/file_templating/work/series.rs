use askama::Template;
use crate::html::types::{WorkSeries, WorkStruct};

#[derive(Template)]
#[template(path = "work/series.html")]
pub struct SeriesTemplate <'a> {
    pub series: &'a WorkSeries,
    pub works: &'a Vec<WorkStruct>,
}