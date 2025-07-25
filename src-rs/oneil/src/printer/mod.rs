mod ast;
mod error;
mod ir;
mod util;

use std::{
    io::{self, Error as IoError, Write},
    path::Path,
};

use oneil_ast::Model as AstModel;
use oneil_ir::model::ModelCollection as IrModelCollection;
use oneil_model_loader::ModelErrorMap;
use oneil_parser::error::ParserError;

pub use util::ColorChoice;

use crate::file_parser::{DoesNotExistError, LoadingError};

pub struct Printer<'a, W>
where
    W: Write,
{
    color_choice: ColorChoice,
    print_debug: bool,
    writer: &'a mut W,
}

impl<'a, W> Printer<'a, W>
where
    W: Write,
{
    pub fn new(use_colors: bool, print_debug: bool, writer: &'a mut W) -> Self {
        let color_choice = if use_colors {
            ColorChoice::EnableColors
        } else {
            ColorChoice::DisableColors
        };

        Self {
            color_choice,
            print_debug,
            writer,
        }
    }

    pub fn print_ast(&mut self, ast: &AstModel) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.writer, "AST: {:?}", ast)?;
        } else {
            ast::print(ast, self.writer)?;
        }

        Ok(())
    }

    pub fn print_ir(&mut self, ir: &IrModelCollection) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.writer, "IR: {:?}", ir)?;
        } else {
            ir::print(ir, self.writer)?;
        }

        Ok(())
    }

    pub fn print_file_error(&mut self, path: &Path, error: &IoError) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.writer, "File error: {:?}", error)?;
        } else {
            error::file::print(path, error, &self.color_choice, self.writer)?;
        }

        Ok(())
    }

    pub fn print_parser_errors(&mut self, path: &Path, errors: &[ParserError]) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.writer, "Parser error: {:?}", errors)?;
        } else {
            error::parser::print_all(path, errors, &self.color_choice, self.writer)?;
        }

        Ok(())
    }

    pub fn print_loader_error(
        &mut self,
        error_map: &ModelErrorMap<LoadingError, DoesNotExistError>,
    ) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.writer, "Loader error: {:?}", error_map)?;
        } else {
            error::loader::print(error_map, self.writer)?;
        }

        Ok(())
    }
}
