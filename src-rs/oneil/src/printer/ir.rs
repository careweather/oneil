use oneil_ir::model::ModelCollection;

/// Prints the IR in a hierarchical tree format for debugging
pub fn print(ir: &ModelCollection, print_debug: bool) {
    if print_debug {
        println!("IR: {:?}", ir);
        return;
    }

    println!("ModelCollection");
    println!("├── Python Imports:");

    // Print Python imports (this is the only public method available)
    let python_imports = ir.get_python_imports();
    if python_imports.is_empty() {
        println!("│   └── [none]");
    } else {
        for (i, import) in python_imports.iter().enumerate() {
            let is_last = i == python_imports.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };
            println!("│   {}Import: \"{}\"", prefix, import.as_ref().display());
        }
    }

    println!("└── Models: [Use --print-debug to see full details]");
    println!();
    println!("Note: The IR tree format is limited due to private fields in the API.");
    println!(
        "Use --print-debug to see the complete IR structure with all models, parameters, and expressions."
    );
}
