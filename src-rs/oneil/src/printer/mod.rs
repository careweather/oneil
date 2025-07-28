mod ast;
mod error;
mod ir;
mod util;

pub use util::ColorChoice;

use std::io::{self, Write};

use oneil_ast::Model as AstModel;
use oneil_ir::model::ModelCollection as IrModelCollection;

use crate::convert_error::Error;

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

    pub fn print_error(&mut self, error: &Error) -> io::Result<()> {
        if self.print_debug {
            writeln!(self.writer, "Error: {:?}", error)?;
        } else {
            error::print(error, &self.color_choice, self.writer)?;
        }

        Ok(())
    }

    pub fn print_errors(&mut self, errors: &[Error]) -> io::Result<()> {
        for error in errors {
            self.print_error(error)?;
            writeln!(self.writer)?;
        }

        Ok(())
    }
}
