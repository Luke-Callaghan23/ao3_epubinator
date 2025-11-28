use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;


pub fn create_epub_zip_file (out_name: &String, out_dir_path: &Path) {
    let zip_name = Path::new("..").join(format!("{out_name}.epub"));
    if fs::exists(&zip_name).expect("Error checking zip existence") {
        fs::remove_file(&zip_name).expect("Error removing existing zip");
    }

    print!("Creating zip of ePub contents . . . ");
    std::io::stdout().flush().expect("Failed to flush stdout"); 

    let original_dir = std::env::current_dir().expect("Getting working directory");

    // All of the epub files were written relative to the output directory specified by the user
    std::env::set_current_dir(out_dir_path).expect("Error changing current dir to staging directory");

    // First zip command to zip the mimetype file into the epub
    // mimetype must be the first file in the archive, and it must have 0 compression on it for epub readers
    //      to be able to read it
    // Ask me how long it took me to figure that one out :)
    let mut zip_initial_command = Command::new("zip");
    zip_initial_command.arg("-X0");                                             
    zip_initial_command.arg(&zip_name);
    zip_initial_command.arg("mimetype");
    zip_initial_command.output().expect("Zip (initial) execution");

    // All others can be packed in any order with max compression
    let mut zip_remaining_command = Command::new("zip");
    zip_remaining_command.arg("-Xr9D");
    zip_remaining_command.arg(&zip_name);
    zip_remaining_command.arg("content.opf");
    zip_remaining_command.arg("toc.ncx");
    zip_remaining_command.arg("page_styles.css");
    zip_remaining_command.arg("stylesheet.css");
    zip_remaining_command.arg("toc_sheet.css");
    zip_remaining_command.arg("META-INF/");
    zip_remaining_command.arg("META-INF/*");
    zip_remaining_command.arg("indexes/");
    zip_remaining_command.arg("indexes/*");
    zip_remaining_command.arg("indexes/**/*");
    zip_remaining_command.arg("content/");
    zip_remaining_command.arg("content/*");
    zip_remaining_command.arg("content/**/*");
    zip_remaining_command.output().expect("Zip (remaining) execution");

    println!("Done.");

    std::env::set_current_dir(original_dir).expect("Error changing current dir to original working directory");
}

// zip -X0 AO3.epub mimetype content.opf toc.ncx page_styles.css stylesheet.css toc_sheet.css META-INF/ META-INF/* indexes/ indexes/* indexes/**/* content/ content/* content/**/*