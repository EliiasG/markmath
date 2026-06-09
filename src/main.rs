use std::env;
use std::path::Path;
use markmath::{run, CompileMode};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 3);
    //run(CompileMode::Live, Path::new(&args[0]), Path::new(&args[1])).unwrap();
    let compile_mode = if args.contains(&"--live".to_string()) {
        CompileMode::Live
    } else if args.contains(&"--no-resolve".to_string()) {
        CompileMode::NonResolving
    } else {
        CompileMode::Resolving
    };
    println!("{}", args.join(" "));
    run(compile_mode, Path::new(&args[1]), Path::new(&args[2])).unwrap();
}
