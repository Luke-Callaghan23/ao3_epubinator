use askama::Template;

#[derive(Template)]
#[template(path = "content_opf.html")]
pub struct ContentOpf {
    pub output_name: String,
    pub xhtmls: Vec<String>
}

impl ContentOpf {
    pub fn new(output_name: String, xhtmls: &Vec<String>) -> Self {

        // Remove "output_name/" from the beginning of each xhtml path
        // These paths are created relative to the working directory, but when we add them
        //      to content.opf, they need to relative to the output_name directory
        let mut xhtml_beginning = output_name.clone();
        xhtml_beginning.push_str("/");

        let repl: Vec<String> =  xhtmls.iter().map(| xhtml | {
            xhtml.replace(&xhtml_beginning, "")
        }).collect();

        ContentOpf { 
            output_name: output_name, 
            xhtmls: repl
        }
    }
}