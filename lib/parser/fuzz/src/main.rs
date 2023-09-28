use hongg::fuzz;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            if let Ok(text) = core::str::from_utf8(data) {
                for _ in trax_parser::Tokenizer::from(text) {}
            }
        });
    }
}
