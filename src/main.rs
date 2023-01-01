use std::io::BufRead;
mod tokenizer;
use std::path::PathBuf;
use std::io;

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

fn tokenize_lines<I>(t: Tokenizer<char>, lines: I)
where I: Iterator<Item = String>,
{

    for line in lines {
        //dbg!(line);
        let tokens = t.tokenize(&line);
        dbg!(&tokens);
        dbg!(&tokens.len());

        //Sanity check TODO Remove?
        let text = t.detokenize(tokens);
        //dbg!(&text);
        assert!(&text == &line);
    }
}


fn main() {
    let cli = Args::parse();

    let encoder_filename = cli.encoder_filename.unwrap_or("encoder.json".into());
    let vocab_filename = cli.vocab_filename.unwrap_or("vocab.bpe".into());

    let my_tokenizer: Tokenizer<char> = Tokenizer::new(encoder_filename, vocab_filename);

    if cli.input.is_some() {
        let lines = cli.input.into_iter();
        tokenize_lines(my_tokenizer, lines);
    } else {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().map(|l| l.unwrap());
        tokenize_lines(my_tokenizer, lines);
    };
}
