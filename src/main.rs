mod tokenizer;
use tokenizer::Tokenizer;

fn main() {
    let my_tokenizer: Tokenizer<char> = Tokenizer::new("encoder.json", "vocab.bpe");
    let test_text = "This is a test! y'all's alright?\nDo newlines work?!%? 1535";
    let tokens = my_tokenizer.tokenize(test_text);
    dbg!(&tokens);
    dbg!(&tokens.len());
    let text = my_tokenizer.detokenize(tokens);
    dbg!(&text);
    assert!(&text == &test_text);
}
