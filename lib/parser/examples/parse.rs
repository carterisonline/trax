use std::env;
use std::fs;
use std::io::Read;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        println!("Usage: parse file.xml");
        return;
    }

    let text = load_file(&args[1]);

    if let Err(e) = parse(&text) {
        println!("Error: {}.", e);
    }
}

fn parse(text: &str) -> Result<(), trax_parser::Error> {
    println!("{}", trax_parser::Tokenizer::from(text));
    // for token in trax_parser::Tokenizer::from(text) {
    //     println!("{:#?}", token?);
    // }

    Ok(())
}

fn load_file(path: &str) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}
