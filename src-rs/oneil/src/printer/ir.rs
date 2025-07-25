use std::io::{self, Write};

use oneil_ir::model::ModelCollection;

/// Prints the IR in a hierarchical tree format for debugging
pub fn print(ir: &ModelCollection, writer: &mut impl Write) -> io::Result<()> {
    writeln!(writer, "ModelCollection")?;
    writeln!(writer, "├── Python Imports:")?;

    // Print Python imports (this is the only public method available)
    let python_imports = ir.get_python_imports();
    if python_imports.is_empty() {
        writeln!(writer, "│   └── [none]")?;
    } else {
        for (i, import) in python_imports.iter().enumerate() {
            let is_last = i == python_imports.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            writeln!(
                writer,
                "│   {}Import: \"{}\"",
                prefix,
                import.as_ref().display()
            )?;
        }
    }

    writeln!(
        writer,
        "└── Models: [Use --print-debug to see full details]"
    )?;
    writeln!(writer)?;
    writeln!(
        writer,
        "Note: The IR tree format is limited due to private fields in the API."
    )?;
    writeln!(
        writer,
        "Use --print-debug to see the complete IR structure with all models, parameters, and expressions."
    )?;

    Ok(())
}
