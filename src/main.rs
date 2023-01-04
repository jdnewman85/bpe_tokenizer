use std::io::BufRead;
mod tokenizer;
use std::path::PathBuf;
use std::{fs, io};

use tokenizer::Tokenizer;

use clap::Parser;

#[macro_use] extern crate rocket;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    input: Option<String>,

    #[arg(short, long, value_name = "Input File")]
    input_filename: Option<PathBuf>,

    #[arg(short, long, value_name = "default: encoder.json")]
    encoder_filename: Option<PathBuf>,
    #[arg(short, long, value_name = "default: vocab.bpe")]
    vocab_filename: Option<PathBuf>,

    #[arg(short, long)]
    serve: bool, //TODO IP/Port
}

#[get("/")]
fn index() -> &'static str {
    "Hello, wolrd!"
}

//#[launch]
//fn rocket() -> _ {
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let cli = Args::parse();

    let encoder_filename = cli.encoder_filename.unwrap_or("encoder.json".into());
    let vocab_filename = cli.vocab_filename.unwrap_or("vocab.bpe".into());

    let my_tokenizer: Tokenizer<char> = Tokenizer::new(encoder_filename, vocab_filename);

    if cli.serve {
        let _rocket = rocket::build().mount("/", routes![index]).launch().await?;
    } else if cli.input.is_some() {
        let input = cli.input.unwrap();
        //dbg!(&input);
        let tokens = my_tokenizer.tokenize(input);
        dbg!(&tokens);
        dbg!(tokens.len());
    } else if let Some(input_filename) = cli.input_filename {
        //TODO Stream in?
        let input = fs::read_to_string(input_filename).unwrap();
        //dbg!(&input);
        let tokens = my_tokenizer.tokenize(input);
        dbg!(&tokens);
        dbg!(tokens.len());
    } else {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().map(|l| l.unwrap());
        my_tokenizer.tokenize_lines(lines);
    };

    Ok(())
}
