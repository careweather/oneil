mod ast;
mod error;
mod ir;
mod util;

use std::{io::Error as IoError, path::Path};

use oneil_ast::Model as AstModel;
use oneil_ir::model::ModelCollection as IrModelCollection;
use oneil_model_loader::ModelErrorMap;
use oneil_parser::error::ParserError;

pub use util::ColorChoice;

use crate::file_parser::{DoesNotExistError, FileLoader, LoadingError};

pub struct Printer {
    color_choice: ColorChoice,
    print_debug: bool,
}

impl Printer {
    pub fn new(use_colors: bool, print_debug: bool) -> Self {
        let color_choice = if use_colors {
            ColorChoice::EnableColors
        } else {
            ColorChoice::DisableColors
        };

        Self {
            color_choice,
            print_debug,
        }
    }

    pub fn print_ast(&self, ast: &AstModel) {
        if self.print_debug {
            println!("AST: {:?}", ast);
        } else {
            ast::print(ast);
        }
    }

    pub fn print_ir(&self, ir: &IrModelCollection) {
        ir::print(ir);
    }

    pub fn print_file_error(&self, path: &Path, error: &IoError) {
        if self.print_debug {
            println!("File error: {:?}", error);
        } else {
            error::file::print(path, error, &self.color_choice);
        }
    }

    pub fn print_parser_error(&self, path: &Path, errors: &[ParserError]) {
        if self.print_debug {
            println!("Parser error: {:?}", errors);
        } else {
            error::parser::print(path, errors);
        }
    }

    pub fn print_loader_error(&self, error_map: &ModelErrorMap<LoadingError, DoesNotExistError>) {
        if self.print_debug {
            println!("Loader error: {:?}", error_map);
        } else {
            error::loader::print(error_map);
        }
    }
}
