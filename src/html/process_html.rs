use lazy_static::lazy_static;
use std::{fs::{read_dir, read_to_string}, io::Error, path::Path};

use regex::Regex;
use scraper::{Html, Selector, ElementRef};
use crate::html::{sanitize_html::sanitize_html, types::*};

fn element_ref_next_element_sibling <'a> (elt: ElementRef<'a>) -> Option<ElementRef<'a>> {
    elt.next_siblings().find(| sibling | {
        if let Some(_sibling) = ElementRef::wrap(*sibling) {
            return true;
        }
        return false;
    }).and_then(| node | {
        ElementRef::wrap(node)
    })
}

fn process_single_chapter (header_elt: ElementRef<'_>) -> Chapter {
    let title = header_elt.inner_html();
    return finish_chapter(0, title, None, header_elt);
}

fn process_multi_chapter (order: usize, meta_group_elt: ElementRef<'_>) -> Chapter {
    lazy_static! {
        static ref header_selector: Selector = Selector::parse("h2.heading").unwrap();
        static ref paragraph_selector: Selector  = Selector::parse("p").unwrap();
    }

    let title = meta_group_elt.select(&header_selector).next().unwrap().inner_html();
    let mut summary: Option<String> = None;
    for meta_child_elt in meta_group_elt.select(&paragraph_selector) {
        if meta_child_elt.inner_html() == "Chapter Summary" {
            summary = Some(
                String::from(
                    element_ref_next_element_sibling(meta_child_elt)
                        .unwrap()
                        .inner_html()
                        .trim()
                )
            );
        }
    }
    return finish_chapter(order, title, summary, meta_group_elt);
}


fn finish_chapter (order: usize, title: String, summary: Option<String>, elt: ElementRef<'_>) -> Chapter {
    let userstuff = element_ref_next_element_sibling(elt).unwrap();

    let data = userstuff.inner_html();
    return Chapter {
        playback_id: 0,
        order: order,
        title: String::from(title.trim()),
        summary: sanitize_html(summary.unwrap_or(String::from("No Summary"))),
        data: sanitize_html(String::from(data.trim()))
    }
}




fn process_anchors <'a> (dt: ElementRef<'a>) -> Vec<Anchor> {
    lazy_static! {
        static ref anchor_selector: Selector = Selector::parse("a").unwrap();
    };

    let mut anchors: Vec<Anchor> = Vec::new();
    let anchor_elts = element_ref_next_element_sibling(dt)
        .unwrap()
        .select(&anchor_selector);

    for anchor_elt in anchor_elts {
        anchors.push(Anchor {
            link: String::from(anchor_elt.attr("href").unwrap().trim()),
            name: String::from(anchor_elt.inner_html().trim()),
        })
    }
    return anchors;
}

struct WorkAnchor <'a> {
    regex: &'a Regex,
    vec: Option<Vec<Anchor>>
}


fn process_html (doc: Html, id: usize) -> Work {
    lazy_static! {
        static ref title_selector:                   Selector = Selector::parse("p.message b").unwrap();
        static ref link_selector:                    Selector = Selector::parse("p.message a:nth-of-type(2)").unwrap();
        static ref tag_container_selector:           Selector = Selector::parse("dl.tags").unwrap();
        static ref summary_selector:                 Selector = Selector::parse("div.meta blockquote.userstuff").unwrap();
        static ref author_elt_selector:              Selector = Selector::parse("a[rel=\"author\"").unwrap();
        static ref single_chapter_header_selector:   Selector = Selector::parse("#chapters > h2").unwrap();
        static ref multi_chapters_headers_selector:  Selector = Selector::parse("#chapters > div.meta.group").unwrap();
        
        static ref fandoms_regex:          Regex = Regex::new("Fandoms?:").unwrap();
        static ref relationships_regex:    Regex = Regex::new("Relationships?:").unwrap();
        static ref characters_regex:       Regex = Regex::new("Characters?:").unwrap();
        static ref additional_tags_regex:  Regex = Regex::new("Additional Tags?:").unwrap();

        static ref part_regex:             Regex = Regex::new(r"Part (?<part>\d+) of").unwrap();
        static ref wc_regex:               Regex = Regex::new(r"Words: (?<wc>[\d,]+)").unwrap();
    }

    let title = doc.select(&title_selector).next().unwrap().inner_html();
    let link = doc.select(&link_selector).next().unwrap().inner_html();
    
    let mut search: [WorkAnchor; 4] = [
        WorkAnchor { regex: &fandoms_regex, vec: None },
        WorkAnchor { regex: &relationships_regex, vec: None },
        WorkAnchor { regex: &characters_regex, vec: None },
        WorkAnchor { regex: &additional_tags_regex, vec: None }
    ];

    let mut series: Option<Series> = None;
    let mut wc: Option<String> = None;

    let tag_container = doc.select(&tag_container_selector).next().unwrap();
    for tag_container_child in tag_container.child_elements() {
        for index in 0usize..search.len() {
            let anchor: &mut WorkAnchor = &mut search[index];
            if anchor.regex.is_match(&tag_container_child.inner_html()[..]) {
                anchor.vec = Some(process_anchors(tag_container_child));
            }
        }

        if tag_container_child.inner_html() == "Series: " {
            let anchor = process_anchors(tag_container_child).into_iter().next().unwrap();

            let part_text = element_ref_next_element_sibling(tag_container_child).unwrap().inner_html();
            let part_number_str = part_regex
                .captures(&part_text)
                .and_then(| a | a.name("wc"))
                .and_then(| mt | Some(mt.as_str()))
                .unwrap()
            ;

            series = Some(Series {
                name: anchor.name,
                link: anchor.link,
                part_number: part_number_str.parse().unwrap()
            })
        }

        if tag_container_child.inner_html() == "Stats:" {
            let stats = element_ref_next_element_sibling(tag_container_child).unwrap().inner_html();
            wc = wc_regex.captures(&stats[..])
                .and_then(| cap | cap.name("wc"))
                .and_then(| mt | Some(String::from(mt.as_str())));
        }
    }

    let summary = match doc.select(&summary_selector).next() {
        Some(summary) => String::from(summary.inner_html().trim()),
        None => String::from("No Summary"),
    };

    let author_elt = doc.select(&author_elt_selector).next();
    let author = match author_elt {
        Some(author_elt) => {
            Author {
                link: String::from(author_elt.attr("href").unwrap().trim()),
                name: String::from(author_elt.inner_html().trim())
            }
        },
        None => {
            Author {
                link: String::from("https://archiveofourown.org"),
                name: String::from("Anonymous")
            }
        },
    };

    let single_chapter_header_opt = doc.select(&single_chapter_header_selector).next();
    let multi_chapter_headers = doc.select(&multi_chapters_headers_selector);

    let mut chapters: Vec<Chapter> = Vec::new();
    if let Some(single_chapter_header) = single_chapter_header_opt {
        chapters.push(process_single_chapter(single_chapter_header));
    }
    else {
        for (index, chpater_header_elt) in multi_chapter_headers.enumerate() {
            chapters.push(process_multi_chapter(index, chpater_header_elt))
        }
    }

    
    return Work {
        id: id,
        playback_id: 0,
        title,
        link,
        fandoms: std::mem::take(&mut search[0].vec).unwrap_or(vec![]),
        relationships: std::mem::take(&mut search[1].vec).unwrap_or(vec![]),
        characters: std::mem::take(&mut search[2].vec).unwrap_or(vec![]),
        tags: std::mem::take(&mut search[3].vec).unwrap_or(vec![]),
        series,
        wc: wc.unwrap_or(String::from("Unknown")),
        summary: sanitize_html(summary),
        author,
        chapters,
    };
}

#[allow(unused_parens)]
pub fn process_ao3_htmls (root: &str) -> Result<Vec<Work>, Error> {
    let path = Path::new(root);
    let entries = match read_dir(&path) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Error reading directory {}: {err}", path.as_os_str().display());
            return Err(err);
        },
    };

    Ok(
        entries.enumerate().filter_map(| (index, entry) | {
            let dirent = match entry {
                Ok(dirent) => dirent,
                Err(err) => {
                    eprintln!("Could not read dir entry at index {index} from {} (Skipping): {err}", path.as_os_str().display());
                    return None;
                },
            };
        
            match dirent.metadata() {
                Ok(metadata) => {
                    if (
                        dirent.file_name().as_os_str().to_str().unwrap().ends_with(".html") 
                        && (metadata.is_file() || metadata.is_symlink())
                    ) {
                        Some(dirent.path())
                    }
                    else { None }
                },
                Err(err) => {
                    eprintln!("Could not read metadata of dir entry at index {index} from {} (Skipping): {err}", path.as_os_str().display());
                    return None;
                }
            }
        })
        .enumerate()
        .filter_map(| (index, path) | {
            let doc_str = match read_to_string(&path) {
                Ok(doc_str) => doc_str,
                Err(err) => {
                    eprintln!("Error reading file {} (skipping): {err}", path.as_os_str().display());
                    return None;
                },
            };
    
            let doc = Html::parse_document(&doc_str[..]);
            Some(process_html(doc, index))
        })
        .collect()
    )
}


#[allow(unused)]
pub fn process_ao3_html (html_path: &str) -> Result<Work, Error> {
    let path = Path::new(html_path);
    let doc_str = match read_to_string(&path) {
        Ok(doc_str) => doc_str,
        Err(err) => {
            eprintln!("Error reading file {} (skipping): {err}", path.as_os_str().display());
            return Err(err);
        },
    };

    let doc = Html::parse_document(&doc_str[..]);
    Ok(process_html(doc, 0))
}