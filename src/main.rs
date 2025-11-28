
mod html;
mod epub;

use std::process::exit;
use std::{env, io};
use std::io::{stdin,stdout};
use std::io::Write;
use structopt::StructOpt;
use std::{path::Path, process::Command, fs};


fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}


#[derive(Debug, StructOpt)]
#[structopt(name = "AO3 Epubinator")]
struct Opt {
    #[structopt(short, long, help="Directory containing AO3 HTML files to ingest.")]
    dir: String,

    #[structopt(short, long, help="Directory to write ePub files into.  The basename of this directory will also be used as the file name of the resulting ePub.  NOTE: THIS SHOULD POINT TO A NEW DIRECTORY THAT DOESN'T EXIST.  IF THE DIRECTORY EXISTS, YOU WILL BE PROMPTED TO DELETE IT!")]
    output: String,

    #[structopt(short = "k", long = "keep_staging_dir", help="Flag to keep the ePub file staging directory specified by --output argument.  Default is false since it mostly just takes up space after the ePub is generated.")]
    keep_staging_dir: bool
}


fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();
    let root = opt.dir;
    let out_name = opt.output;
    let keep_staging_dir = opt.keep_staging_dir;

    let program_name = env::args().next().unwrap();

    let out_dir_path = Path::new(&out_name);
    if fs::exists(out_dir_path).expect("Checking existence") {
        let mut input=String::new();

        print!("WARNING: Output directory specified by '--output' option already exists.  \n'{program_name}' expects directory --output directory (you put '{}') to not exist.\nIs it okay to delete {} before continuing? [y/n]: ", out_dir_path.to_str().unwrap(), out_dir_path.to_str().unwrap());
        stdout().flush().expect("Failed to flush stdout");

        stdin().read_line(&mut input).expect("Failed to read user input");
        let input = input.trim();
        if input.len() == 1 && input.chars().next().unwrap() == 'y' {
            print!("Deleting old data for ePub . . . ");
            std::io::stdout().flush().expect("Failed to flush stdout"); 
            fs::remove_dir_all(out_dir_path).expect("Deleting directory");
            println!("Done.");
        }
        else {
            println!("You entered '{input}' which does not match 'y'.  Exiting . . . ");
            exit(1);
        }
    }

    print!("Copying template data for ePub . . . ");
    std::io::stdout().flush().expect("Failed to flush stdout"); 
    let copy_dir_path = Path::new("copy_dir");
    copy_dir_all(copy_dir_path, out_dir_path).expect("Copying directory");
    println!("Done.");
    
    print!("Ingesting AO3 HTMLs . . . ");
    std::io::stdout().flush().expect("Failed to flush stdout"); 
    let works = html::process_html::process_ao3_htmls(&root[..]).expect("Works ingestion failed");
    println!("Done.");

    print!("Writing epub files . . . ");
    std::io::stdout().flush().expect("Failed to flush stdout"); 
    epub::write_epub_files::write_epub_files(out_dir_path, &out_name, works);
    println!("Done.");

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
    if !keep_staging_dir {
        print!("Removing staging directory of ePub files . . . ");
        std::io::stdout().flush().expect("Failed to flush stdout"); 
        fs::remove_dir_all(out_dir_path).expect("Deleting directory");
        println!("Done.");
    }
    
    Ok(())
}