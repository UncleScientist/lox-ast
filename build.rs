use std::io;
mod generate_ast;
use generate_ast::*;

fn main() -> io::Result<()> {
    generate_ast("src")
}
