use std::{env, fs, io::Read};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        println!("Usage: new_document file.trax");
        return;
    }

    let text = load_file(&args[1]);

    if let Err(e) = new_document(&text) {
        println!("Error: {}.", e);
    }
}

fn new_document(text: &str) -> Result<(), trax_document::DocumentParseError> {
    let document = trax_document::Document::new(text)?;
    println!("{:#?}", document);
    Ok(())
}

fn load_file(path: &str) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}
