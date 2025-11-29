use std::{collections::HashMap, fs, path::Path};
use crate::{epub::file_templating::{category_index::{CategoryIndex, CategoryListing}, category_listing_index::CategoryListingIndex, content_opf::ContentOpf, index_index::IndexIndex, toc::TableOfContents, work::{chapter::WorkChapter, introduction::WorkIntroduction, preview::WorkPreview, series::SeriesTemplate}, works_index::WorksIndex}, html::types::{Anchor, Category, Work, WorkSeries, WorkStruct}};

pub struct EpubWriter {
    // Every time we write an xhtml to the staging directory, we need to track that xhtml path
    //      because content.opf will want a full log of all xhtmls in the ePub
    // content.opf also specifies the order in which xhtmls will appear in the ePub,
    //      so the order the xhtml paths get inserted into this list is also the order of the content
    //      of this ePub
    all_xhtmls: Vec<String>
}

impl EpubWriter {

    pub fn new () -> Self {
        EpubWriter {
            all_xhtmls: Vec::new()
        }
    }

    // Takes an askama template, writes it to the desired path, and expects all errors
    // SIDE EFFECT: if the write path is an xhtml, the path will be added to `&mut all_xhtmls`
    fn render_and_write <T: askama::Template> (&mut self, path: &Path, template: T) {
        fs::write(&path, template
            .render()
            .expect(
                &format!("Error rendering template for {}", path.to_str().unwrap())[..]
            )
            .to_string()
        ).expect(
            &format!("Error writing rendered template for {}", path.to_str().unwrap())[..]
        );
    
        // Only add to the xhtmls vector if it is a .xhtml file
        let path_str = String::from(path.to_str().unwrap());
        if path_str.ends_with(".xhtml") {
            self.all_xhtmls.push(path_str);
        }
    }

    fn assign_playback_to_work_struct (running_play_order: &mut usize, work: &mut WorkStruct) {
        work.playback_id = *running_play_order;
        *running_play_order += 1;

        for chapter in &mut work.chapters {
            chapter.playback_id = *running_play_order;
            *running_play_order += 1;
        }
    }

    fn write_work_struct (&mut self, work: &WorkStruct, out_dir_path: &Path, category_listings: &HashMap<Category, HashMap<String, CategoryListing>>, series: Option<(&WorkSeries, &Vec<WorkStruct>)>) {
        // Make the folder where all the content for this work will be storeds
        let work_content_path = out_dir_path.join("content").join(
            format!("work-{}", work.id)
        );
        fs::create_dir(&work_content_path).expect(
            &format!("Creating content directory for work {}", work.id)[..]
        );

        // Work Introduction -> 
        //      Listing of all categories and subcategories in this work
        self.render_and_write(
            &work_content_path.join(format!("work-{}.xhtml", work.id)), 
            WorkIntroduction::new(&work, &category_listings, series)
        );

        // Work preview -> Summary
        self.render_and_write(
            &work_content_path.join(format!("work-{}-preview.xhtml", work.id)), 
            WorkPreview { work: &work }
        );

        // Chapters -> 
        //      Actual content of the work
        for chapter in work.chapters.iter() {
            self.render_and_write(
                &work_content_path.join(format!("work-{}-chapter-{}.xhtml", work.id, chapter.order)), 
                WorkChapter {  
                    work_author: &work.author,
                    work_title: &work.title,
                    chapter: &chapter,
                }
            );
        }
    }

    pub fn write_epub_files(&mut self, out_dir_path: &Path, out_name: &str, categories: &[Category], mut works: Vec<Work>) {
        // Assign correct playback ids to the works
        // Used in the table of contents page
        // Impossible to do within askama itself, so they need to be pre-computed
        let mut running_play_order = 0;
        for work in &mut works {
            match work {
                Work::Single(work_struct) => {
                    EpubWriter::assign_playback_to_work_struct(&mut running_play_order, work_struct);
                },
                Work::Series(ser, works) => {
                    ser.playback_id = running_play_order;
                    running_play_order += 1;

                    works.iter_mut().for_each(| work_struct | {
                        EpubWriter::assign_playback_to_work_struct(&mut running_play_order, work_struct);
                    });
                },
            }
        }
    
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
    
        // indexes/index_index.xhtml -> 
        //      Index of the categories
        self.render_and_write(
            &indexes_path.join("index_index.xhtml"), 
            IndexIndex {
                output_name: String::from(out_name),
                categories: &categories,
            }
        );
    
        // indexes/work_index.xhtml -> 
        //      Index of all works and all of their chapters
        self.render_and_write(
            &indexes_path.join("works_index.xhtml"), 
            WorksIndex {
                output_name: String::from(out_name),
                categories: &categories,
                works: &works,
            }
        );

        let mut work_structs: Vec<&WorkStruct> = Vec::new();
        for work in &works {
            match work {
                Work::Single(ws) => work_structs.push(ws),
                Work::Series(_, works) => works.iter().for_each(| ws | {
                    work_structs.push(ws);
                }),
            }
        }
    
        for category in &*categories {
            // Accumulate listing of subcategories for each category
            // Iterate over all the works and get all subcategories inside of this category
            //      and store them as a CategoryListing
            // When more than one work shares a category/sub-category, then add that work to the list
            //      of works in that CategoryListing
            // Accumulate all information in this hashmap
            // Key is the link to the subcategory (subcategories are always of `Anchor` struct type, so they all have a link and a name)
            // Value is the accumulated list of all works under the category/subcategory combination
            let mut listings: HashMap<String, CategoryListing> = HashMap::new();

            for work in &work_structs {
    
                // Extract subcategory list from this work for this category
                let work_category_entries = match category {
                    // Titles need to created on the spot using the work data itself
                    Category::Titles => &vec![Anchor { link: work.link.clone(), name: work.title.clone() }],
                    // Since there is only one author, just clone the author element and store inside of a vec
                    // (Because the rest of the function expects a vec)
                    // (Need to clone the author element because the expected type is Vec<Anchor>, not Vec<&Anchor>)
                    Category::Authors => &vec![ work.author.clone() ],

                    _ => work.category_data.get(&category).unwrap(),
                };
    
                // Check every subcategory in this work/category combination
                for work_category_entry in work_category_entries {
                    // If this subcategory was found already, add the work the accumulating list 
                    if let Some(existing_listing) = listings.get_mut(&work_category_entry.link) {
                        existing_listing.count += 1;
                        existing_listing.works.push(&work);
                    }
                    // Otherwise create a new CategoryListing object for the subcategory
                    else {
                        listings.insert(work_category_entry.link.clone(), CategoryListing { 
                            id: listings.len(), 
                            name: work_category_entry.name.clone(), 
                            count: 1,
                            works: vec![ &work ]
                        });
                    }
                }
            }
    
            // Once all subcategory listings have been accumulated in the hashmap, translate
            //      the hashmap into a list of just (references to) the values in the map
            let mut listing_info: Vec<&CategoryListing<'_>> = listings
                .values()
                .collect();
    
            // Then sort all the subcategories by how many works were in that subcategory, descending
            listing_info.sort_by(| a, b | b.count.cmp(&a.count));
    
            // Write the category index
            // Category index (indexes/<category>/index.xhtml) ->
            //      List of all items in that category and the number of works with that item
            //      Ordered by the number of works in the category item, descending
            // Example: Fandoms: Overwatch (100), Supernatural (50), Pokemon (2)
            self.render_and_write(
                &indexes_path.join(format!("{}/index.xhtml", category)), 
                CategoryIndex {
                    category: category.to_string(),
                    categories: listing_info
                }
            );
    
            // Because each title is (most likely) unique, there does not need to be a listing page for that category
            // The title index page will link directly to each work individually
            if *category != Category::Titles {
                for (_, subcategory_listing) in &listings {
    
                    // Write the category listing index
                    // Category listing index (indexes/<category>/<category>-<subcategory_id>-listing.xhtml) ->
                    //      List of all works inside of a subcategory inside of a category
                    //      No particular order
                    // Example: Fandoms -> Pokemon: "Gotta Catch 'Em All", "Who's that Pokemon?"
                    self.render_and_write(
                        &indexes_path.join(format!("{category}/{category}-{}-listing.xhtml", subcategory_listing.id)), 
                        CategoryListingIndex {
                            category: category.to_string(),
                            listing_name: &subcategory_listing.name,
                            listing: &subcategory_listing.works
                        }
                    );
                }
            }
    
            category_listings.insert(category.clone(), listings);
        }
    
        for work in &works {
            match work {
                Work::Series(work_series, work_structs) => {

                    // For series, write the series page
                    self.render_and_write(
                        &out_dir_path.join("content").join("series").join(format!("series-{}.xhtml", work_series.id)), 
                        SeriesTemplate {
                            series: work_series,
                            works: work_structs
                        }
                    );

                    // Then all the works write after it
                    for work_struct in work_structs {
                        self.write_work_struct(work_struct, out_dir_path, &category_listings, Some((work_series, work_structs)));
                    }
                },
                // For single works, just write the work normally
                Work::Single(work_struct) => self.write_work_struct(work_struct, out_dir_path, &category_listings, None),
            }
            
        }
    
        // toc.ncx
        self.render_and_write(
            &out_dir_path.join("toc.ncx"), 
            TableOfContents {
                output_name: String::from(out_name),
                categories: &categories,
                works: &works
            }
        );
    
        self.render_and_write(
            &out_dir_path.join("content.opf"),
            ContentOpf::new(String::from(out_name), &self.all_xhtmls)
        );
    
    }
}


