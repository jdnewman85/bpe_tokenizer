use std::io::BufRead;
mod tokenizer;
use std::path::PathBuf;
use std::io;

use tokenizer::Tokenizer;

use clap::Parser;

use memuse::DynamicUsage;

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

    let byte_encoder = my_tokenizer.byte_encoder.dynamic_usage();
    dbg!(byte_encoder);
    let byte_decoder = my_tokenizer.byte_decoder.dynamic_usage();
    dbg!(byte_decoder);
    let token_encoder = my_tokenizer.token_encoder.dynamic_usage();
    dbg!(token_encoder);
    let token_decoder = my_tokenizer.token_decoder.dynamic_usage();
    dbg!(token_decoder);
    let bpe_ranks = my_tokenizer.bpe_ranks.dynamic_usage();
    dbg!(bpe_ranks);

    let sum = byte_encoder + byte_decoder + token_encoder + token_decoder + bpe_ranks;
    dbg!(sum / 1024);
    dbg!(sum / 1024 / 1024);

    if cli.input.is_some() {
        let lines = cli.input.into_iter();
        tokenize_lines(my_tokenizer, lines);
    } else {
        let stdin = io::stdin();
        let lines = stdin.lock().lines().map(|l| l.unwrap());
        tokenize_lines(my_tokenizer, lines);
    };
}
