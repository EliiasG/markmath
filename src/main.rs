
mod parse;
mod expression;
mod format;

fn main() {
    println!("{}", parse::tokenize("a + i_1").unwrap());
}
