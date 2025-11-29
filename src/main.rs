mod initialize_fs;
mod html;
mod epub;
mod create_zip;

use std::env;
use std::io::Write;
use structopt::StructOpt;
use std::fs;
use std::path::Path;

use crate::html::types::Category;





#[derive(Debug, StructOpt)]
#[structopt(name = "AO3 Epubinator")]
struct Opt {
    #[structopt(short, long, help="Directory containing AO3 HTML files to ingest.")]
    dir: String,

    #[structopt(short, long, help="File name of the output ePub.  No need to add .epub extension.  NOTE: While creating the ePub files will be stored in a staging directory with the same name as this output file name in the directory you run the program.  If a directory with this name already exists, you will be prompted to delete it.")]
    output_file_name: String,

    #[structopt(short = "k", long = "keep_staging_dir", help="Flag to keep the ePub file staging directory specified by --output argument.  Default is false since it mostly just takes up space after the ePub is generated.")]
    keep_staging_dir: bool,

    #[structopt(short = "y", long = "yes", help="Flag to say yess to deleting old staging directory without being prompted")]
    automatically_delete_staging_dir: bool

}


fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();
    let root = opt.dir;
    let keep_staging_dir = opt.keep_staging_dir;
    let automatically_delete_staging_dir = opt.automatically_delete_staging_dir;
    
    let program_name = env::args().next().unwrap();
    
    let categories = [
        Category::Titles,
        Category::Ratings,
        Category::Categories,
        Category::Fandoms,
        Category::Relationships,
        Category::Characters,
        Category::Tags,
        Category::Authors
    ];
    
    // Can't trust that the user didn't enter a path in --output
    // So, first parse the input as a path, take its basename, then parse again as a path
    let out_name = opt.output_file_name.replace(".epub", "");
    let out_name = String::from(Path::new(&out_name).file_name().unwrap().to_str().unwrap());
    let out_dir_path = Path::new(&out_name);
    
    // The ePub has an initial directory structure that needs to initalized before we start writing
    //      custom content (see 'copy_dir' in the root of the repo)
    // Initialize `out_dir_path` with 'copy_dir' contents (programmatically) before continuing
    initialize_fs::initialize_filesystem_for_epub(&program_name, out_dir_path, &categories, automatically_delete_staging_dir);

    // Process AO3 HTML files and store necessary data in internal structure
    print!("Ingesting AO3 HTMLs . . . ");
    std::io::stdout().flush().expect("Failed to flush stdout"); 
    let works = html::process_html::process_ao3_htmls(&root[..]).expect("Works ingestion failed");
    println!("Done.");

    // Write the ePub files to `out_dir_path`
    print!("Writing epub files . . . ");
    std::io::stdout().flush().expect("Failed to flush stdout"); 
    let mut epub_writer = epub::write_epub_files::EpubWriter::new();
    epub_writer.write_epub_files(out_dir_path, &out_name, &categories, works);
    println!("Done.");

    // Zip everything together
    create_zip::create_epub_zip_file(&out_name, out_dir_path);
    
    if !keep_staging_dir {
        print!("Removing staging directory of ePub files . . . ");
        std::io::stdout().flush().expect("Failed to flush stdout"); 
        fs::remove_dir_all(out_dir_path).expect("Deleting directory");
        println!("Done.");
    }
    
    Ok(())
}