// SPDX-License-Identifier: MPL-2.0

mod lex;
mod parse;

use std::fs;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: whale-c <file.c>");
        std::process::exit(2);
    });

    let src = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("failed to read {path}: {e}");
        std::process::exit(2);
    });

    let program = match parse::parse_translation_unit(&src) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("parse error: {e}");
            std::process::exit(1);
        }
    };

    let mut module = match ir::lower_ast::lower_o0(
        &program,
        "x86_64-whale-linux",
        ir::DataLayout::default_64bit_le(),
    ) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("lower error: {e:?}");
            std::process::exit(1);
        }
    };

    ir::zero::pass::run_zero_pass(&mut module);

    if let Err(e) = ir::verifier::verify_module(&module) {
        eprintln!("verify error: {e:?}");
        std::process::exit(1);
    }

    print!("{}", ir::printer::print_module(&module));
}