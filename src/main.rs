
mod parse;
mod expression;

fn main() {
    println!("{}", parse::tokenize("a_1").unwrap());
}
