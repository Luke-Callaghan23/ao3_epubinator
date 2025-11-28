use std::fs;
use std::io::{Write, stdin, stdout};
use std::path::Path;
use std::process::exit;

use crate::html::types::Category;

fn mimetype () -> &'static str {
    "application/epub+zip"
}

fn container_xml () -> &'static str {
r#"
<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="content.opf" media-type="application/oebps-package+xml"/>
    </rootfiles>
</container>
"#
}

fn page_styles_css () -> &'static str {
r#"
@page {
    margin-bottom: 5pt;
    margin-top: 5pt;
}
"#
}

fn stylesheet_css () -> &'static str {
r#"
.byline {
    display: block;
    text-align: center;
}
.calibre {
    display: block;
    font-size: 1em;
    padding-left: 0;
    padding-right: 0;
    margin: 0 5pt;
}
.calibre1 {
    display: block;
}
.calibre2 {
    font-weight: bold;
}
.calibre3 {
    display: block;
    font-weight: bold;
    margin: 0;
}
.calibre4 {
    display: block;
    margin: 0 0 1em 1em;
}
.calibre5 {
    display: block;
    margin: 0 0 0 1em;
}
.calibre6 {
    display: block;
    font-size: 1.41667em;
    font-weight: bold;
    line-height: 1.2;
    text-align: center;
    margin: 0.67em 0;
}
.calibre7 {
    display: block;
    margin: 1em 0;
}
.calibre8 {
    display: block;
}
.calibre9 {
    font-style: italic;
}
.heading {
    display: block;
    font-size: 1.41667em;
    font-weight: bold;
    line-height: 1.2;
    text-align: center;
    margin: 0.83em 0;
}
.message {
    display: block;
    text-align: center;
    margin: 1em 0;
}
.tags {
    display: block;
    margin: 1em 0;
    padding: 0;
    border: currentColor none medium;
}
.toc-heading {
    display: none;
    font-size: 1.41667em;
    font-weight: bold;
    line-height: 1.2;
    margin: 0.83em 0;
}
.userstuff {
    display: block;
    font-family: serif;
    margin: 1em;
    padding: 0;
}
.userstuff1 {
    display: block;
    font-family: serif;
    padding: 0;
}
.userstuff2 {
    display: block;
    font-family: serif;
    margin: 1em 0;
    padding: 0;
}
"#
}

fn toc_sheet_css () -> &'static str {
r#"
/* NAVIGATION */

ol {
    list-style-type: none;
    margin: 0 0 0 2em;
    padding: 0 0 0 0;
}

ol li {
    margin: 0 0 0 0;
    padding: 0 0 0 0;
}

ol li a {
    text-decoration: none;
    color: black;
    background-color: red;
    font-family: sans-serif;
}

#guide {
    display: none;
}
"#
}

pub fn initialize_filesystem_for_epub (program_name: &String, out_dir_path: &Path, categories: &[Category], automatically_delete_staging_dir: bool) {

    // First make sure that the path doesn't exist already
    // ao3_epubinator expects `out_dir_path` to be a staging directory for the program to copy files into and we don't want to collide with
    //      any of the user's files if that directory already exists
    if fs::exists(out_dir_path).expect("Checking existence") {
        
        let mut input = String::new();
        let response = if !automatically_delete_staging_dir {

            // If it exists, prompt the user to delete it
            print!("WARNING: Output directory specified by '--output' option already exists.  \n'{program_name}' expects directory --output directory (you put '{}') to not exist.\nIs it okay to delete {} before continuing? [y/n]: ", out_dir_path.to_str().unwrap(), out_dir_path.to_str().unwrap());
            stdout().flush().expect("Failed to flush stdout");
    
            stdin().read_line(&mut input).expect("Failed to read user input");
            input.trim()
        }
        // If automatically_delete_staging_dir, skip asking the user, and pretend their response was "y"
        else { "y" };

        if response.len() == 1 && response.chars().next().unwrap() == 'y' {
            print!("Deleting old data . . . ");
            std::io::stdout().flush().expect("Failed to flush stdout"); 
            fs::remove_dir_all(out_dir_path).expect("Deleting directory");
            println!("Done.");
        }
        else {
            println!("You entered '{response}' which does not match 'y'.  Exiting . . . ");
            exit(1);
        }
    }

    print!("Creating template data for ePub . . . ");
    std::io::stdout().flush().expect("Failed to flush stdout"); 

    // Closure to make a directory and expect any errors
    let make_dir = | dir: &Path | {
        fs::create_dir(dir).expect(&format!("Error creating {}", dir.to_str().unwrap())[..]);
    };

    // Closure to write a file and expect any errors
    let write_file = | file: &Path, contents: &str | {
        // Trim the contents first
        // For some reason having a leading newline in container.xml causes the whole epub to break 
        // Ask me how long it took me to figure that one out :)
        let contents = contents.trim();
        fs::write(file, contents).expect(&format!("Error writing file {}", file.to_str().unwrap())[..])
    };

    // See 'copy_dir' from the repo
    // Generate and copy all those files (programmatically) in `out_dir_path`
    
    // Main directory
    make_dir(out_dir_path);

    // Content folder
    let content_dir = out_dir_path.join("content");
    make_dir(&content_dir);

    // Indexes folder
    let indexes_dir = out_dir_path.join("indexes");
    make_dir(&indexes_dir);

    // Write a folder in the indexes folder for each category of indexes
    for category in &*categories {
        let category_dir = indexes_dir.join(category.to_string());
        make_dir(&category_dir);
    }

    // META-INF
    let meta_inf_dir = out_dir_path.join("META-INF");
    make_dir(&meta_inf_dir);

    // META-INF/container.xml
    let meta_inf_container_xml_path = meta_inf_dir.join("container.xml");
    write_file(&meta_inf_container_xml_path, container_xml());

    // page_styles.cc
    let page_style_path = out_dir_path.join("page_styles.css");
    write_file(&page_style_path, page_styles_css());

    // stylesheet.css
    let stylesheet_path = out_dir_path.join("stylesheet.css");
    write_file(&stylesheet_path, stylesheet_css());

    // toc_sheet.css
    let toc_sheet_path = out_dir_path.join("toc_sheet.css");
    write_file(&toc_sheet_path, toc_sheet_css());

    // Most importantly, mimetype
    let mimetype_path = out_dir_path.join("mimetype");
    write_file(&mimetype_path, mimetype());

    println!("Done.");
}