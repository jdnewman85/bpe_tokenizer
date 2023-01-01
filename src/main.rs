mod tokenizer;
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

    if let Some(input) = cli.input {
        let my_tokenizer: Tokenizer<char> = Tokenizer::new(encoder_filename, vocab_filename);
        let tokens = my_tokenizer.tokenize(&input);
        dbg!(&tokens);
        dbg!(&tokens.len());

        //Sanity check TODO Remove?
        let text = my_tokenizer.detokenize(tokens);
    //    dbg!(&text);
        assert!(&text == &input);
    } else {
        println!("TODO: Process from stdin");
    }

}
