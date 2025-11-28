use std::{collections::HashMap, fs, path::Path};

use askama::Template;

use crate::{epub::file_templating::{category_index::{CategoryIndex, CategoryListing}, category_listing_index::CategoryListingIndex, content_opf::ContentOpf, index_index::IndexIndex, toc::TableOfContents, work::{chapter::WorkChapter, introduction::WorkIntroduction, preview::WorkPreview}, works_index::WorksIndex}, html::types::{Anchor, Category, Work}};


pub fn write_epub_files(out_dir_path: &Path, out_name: &str, mut works: Vec<Work>) {
    // Assign correct playback ids to the works
    // Used in the table of contents page
    // Impossible to do within askama itself, so they need to be pre-computed
    let mut running_play_order = 0;
    for work in &mut works {
        work.playback_id = running_play_order;
        running_play_order += 1;

        for chapter in &mut work.chapters {
            chapter.playback_id = running_play_order;
            running_play_order += 1;
        }
    }

    
    let categories = [
        Category::Titles,
        Category::Fandoms,
        Category::Relationships,
        Category::Characters,
        Category::Tags,
        Category::Authors
    ];

    let mut all_xhtmls: Vec<String> = Vec::new();

    // Used to lookup links to category index listing pages
    // (Category index pages is the page that's like (for fandoms) "Index > Fandoms:" and lists all the fandoms and counts of works
    //      like "Overwatch (100)", "Supernatural (50)", "Pokemon (2)", etc")
    // (Category index LISTING page is the page that takes each one of those items listed above and lists all the works in the category
    //      like "Index > Fandoms > Pokemon:" might list "Gotta Catch 'Em All" / "Who's that Pokemon" as the works in this epub from that
    //      fandom)
    // In the works preview page, it lists all the tags / fandoms / relationships /etc. of that work, and all the items
    //      in that page should be clickable and link to the category index LISTING page
    let mut category_listings: HashMap<Category, HashMap<String, CategoryListing>> = HashMap::new();

    let indexes_path = out_dir_path.join("indexes");

    let indexes_index_path = indexes_path.join("index_index.xhtml");
    all_xhtmls.push(String::from(indexes_index_path.to_str().unwrap()));

    let indexes_index = IndexIndex {
        output_name: String::from(out_name),
        categories: &categories,
    };
    fs::write(&indexes_index_path, indexes_index.render().unwrap().to_string()).expect("Error writing index_index.xhtml");


    let works_index_path = indexes_path.join("works_index.xhtml");
    all_xhtmls.push(String::from(works_index_path.to_str().unwrap()));

    let works_index = WorksIndex {
        output_name: String::from(out_name),
        categories: &categories,
        works: &works,
    };
    fs::write(&works_index_path, works_index.render().unwrap().to_string()).expect("Error writing works_index.xhtml");

    for category in &categories {
        let mut listings: HashMap<String, CategoryListing> = HashMap::new();

        for work in &works {
            let work_category_entries = match category {
                Category::Titles => &vec![Anchor { link: work.link.clone(), name: work.title.clone() }],
                Category::Fandoms => &work.fandoms,
                Category::Relationships => &work.relationships,
                Category::Characters => &work.characters,
                Category::Tags => &work.tags,
                Category::Authors => &vec![Anchor { link: work.author.link.clone(), name: work.author.name.clone() }]
            };

            for work_category_entry in work_category_entries {
                if let Some(existing_listing) = listings.get_mut(&work_category_entry.link) {
                    existing_listing.count += 1;
                    existing_listing.works.push(&work);
                }
                else {
                    listings.insert(work_category_entry.link.clone(), CategoryListing { 
                        id: listings.len(), 
                        name: work_category_entry.name.clone(), 
                        count: 1,
                        works: vec![ work ]
                    });
                }
            }
        }

        let mut listing_info: Vec<&CategoryListing<'_>> = listings
            .iter()
            .map(| (_, listing) | listing)
            .collect();

        listing_info.sort_by(| a, b | b.count.cmp(&a.count));

        let category_index = CategoryIndex {
            category: category.to_string(),
            categories: listing_info
        };

        let category_index_path = indexes_path.join(format!("{}/index.xhtml", category));
        all_xhtmls.push(String::from(category_index_path.to_str().unwrap()));

        fs::write(category_index_path, category_index.render().unwrap().to_string()).expect(
            &format!("Error writing category index: {category}")[..]
        );

        if *category != Category::Titles {
            for (_, subcategory_listing) in &listings {
                let category_subcategory_path = indexes_path.join(
                    format!("{category}/{category}-{}-listing.xhtml", subcategory_listing.id)
                );
                all_xhtmls.push(String::from(category_subcategory_path.to_str().unwrap()));
    
                let listing_page = CategoryListingIndex {
                    category: category.to_string(),
                    listing_name: &subcategory_listing.name,
                    listing: &subcategory_listing.works
                };
    
                fs::write(category_subcategory_path, listing_page.render().unwrap().to_string()).expect(
                    &format!("Writing category {} listing {} ", category, subcategory_listing.name)[..]
                );
            }
        }

        category_listings.insert(category.clone(), listings);
    }

    for work in &works {

        let work_content_path = out_dir_path.join("content").join(
            format!("work-{}", work.id)
        );
        fs::create_dir(&work_content_path).expect(
            &format!("Creating content directory for work {}", work.id)[..]
        );

        // Work Introduction
        let work_intro_path = work_content_path.join(
            format!("work-{}.xhtml", work.id)
        );
        all_xhtmls.push(String::from(work_intro_path.to_str().unwrap()));

        let intro_page = WorkIntroduction::new(&work, &category_listings);
        fs::write(work_intro_path, intro_page.render().unwrap().to_string()).expect(
            &format!("Writing work {} introduction", work.id)[..]
        );

        // Work preview
        let work_preview_path = work_content_path.join(
            format!("work-{}-preview.xhtml", work.id)
        );
        all_xhtmls.push(String::from(work_preview_path.to_str().unwrap()));

        let preview_page = WorkPreview { work: &work };
        fs::write(work_preview_path, preview_page.render().unwrap().to_string()).expect(
            &format!("Writing work {} introduction", work.id)[..]
        );

        // Chapters
        for (chapter_id, chapter) in work.chapters.iter().enumerate() {
            let chapter_path = work_content_path.join(
                format!("work-{}-chapter-{}.xhtml", work.id, chapter.order)
            );
            all_xhtmls.push(String::from(chapter_path.to_str().unwrap()));

            let chapter_page = WorkChapter {  
                work_author: &work.author,
                work_title: &work.title,
                chapter: &chapter,
            };
            fs::write(chapter_path, chapter_page.render().unwrap().to_string()).expect(
                &format!("Writing work {} chapter {}", work.id, chapter_id)[..]
            );
        }
    }

    let toc = TableOfContents {
        output_name: String::from(out_name),
        categories: &categories,
        works: &works
    };
    
    let toc_file_path = out_dir_path.join("toc.ncx");
    fs::write(&toc_file_path, toc.render().unwrap().to_string()).expect("Issue writing to file");
    

    let content_opf_path = out_dir_path.join("content.opf");
    let content_opf = ContentOpf::new(String::from(out_name), all_xhtmls);
    fs::write(&content_opf_path, content_opf.render().unwrap().to_string()).expect("Error while writing content.opf");
}