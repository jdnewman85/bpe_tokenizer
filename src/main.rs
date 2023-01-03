use std::io::BufRead;
mod tokenizer;
use std::io;
use std::path::PathBuf;

use tokenizer::Tokenizer;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    input: Option<String>,

    #[arg(short, long, value_name = "default: encoder.json")]
    encoder_filename: Option<PathBuf>,
    #[arg(short, long, value_name = "default: vocab.bpe")]
    vocab_filename: Option<PathBuf>,
}

fn main() {
    let cli = Args::parse();

    let encoder_filename = cli.encoder_filename.unwrap_or("encoder.json".into());
    let vocab_filename = cli.vocab_filename.unwrap_or("vocab.bpe".into());

    let my_tokenizer: Tokenizer<char> = Tokenizer::new(encoder_filename, vocab_filename);

    if cli.input.is_some() {
        let lines = cli.input.into_iter();
        my_tokenizer.tokenize_lines(lines);
    } else {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().map(|l| l.unwrap());
        my_tokenizer.tokenize_lines(lines);
    };
}
