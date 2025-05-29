use std::env;
use std::path::Path;
use markmath::{run, CompileMode};

fn main() {
    let args: Vec<String> = env::args().collect();
    //run(CompileMode::Live, Path::new(&args[0]), Path::new(&args[1])).unwrap();
    run(CompileMode::Live, Path::new(&"examples/test.md"), Path::new(&"examples/out.html")).unwrap();
}
