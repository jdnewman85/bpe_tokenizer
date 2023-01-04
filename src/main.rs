use std::io::BufRead;
mod tokenizer;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, io};

use tokenizer::{Tokenizer, TokenizerType};

use clap::Parser;

use rocket::serde::json::Json;
use rocket::State;

#[macro_use]
extern crate rocket;

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

struct TokenizerState<T: TokenizerType> {
    tokenizer_arc: Arc<Tokenizer<T>>,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, wolrd!"
}

#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[get("/tokenize/<input>")]
fn tokenize(input: &str, tokenizer_state: &State<TokenizerState<char>>) -> Json<Vec<u16>> {
    Json(tokenizer_state.tokenizer_arc.tokenize(input))
}

//#[launch]
//fn rocket() -> _ {
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let cli = Args::parse();

    let encoder_filename = cli.encoder_filename.unwrap_or("encoder.json".into());
    let vocab_filename = cli.vocab_filename.unwrap_or("vocab.bpe".into());

    let tokenizer_ref: Tokenizer<char> = Tokenizer::new(encoder_filename, vocab_filename);
    let tokenizer_arc = Arc::new(tokenizer_ref);

    if cli.serve {
        let _rocket = rocket::build()
            .manage(TokenizerState::<char> { tokenizer_arc })
            .mount("/", routes![index, hello, tokenize])
            .launch()
            .await?;
    } else if cli.input.is_some() {
        let input = cli.input.unwrap();
        //dbg!(&input);
        let tokens = tokenizer_arc.tokenize(input);
        dbg!(&tokens);
        dbg!(tokens.len());
    } else if let Some(input_filename) = cli.input_filename {
        //TODO Stream in?
        let input = fs::read_to_string(input_filename).unwrap();
        //dbg!(&input);
        let tokens = tokenizer_arc.tokenize(input);
        dbg!(&tokens);
        dbg!(tokens.len());
    } else {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().map(|l| l.unwrap());
        tokenizer_arc.tokenize_lines(lines);
    };

    Ok(())
}
