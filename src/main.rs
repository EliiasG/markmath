
mod parse;
mod expression;

fn main() {
    println!("{}", parse::tokenize("a + i_1").unwrap());
}
