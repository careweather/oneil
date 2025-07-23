use oneil_ir::model::ModelCollection;

/// Prints the IR in a hierarchical tree format for debugging
pub fn print(ir: &ModelCollection, print_debug: bool) {
    if print_debug {
        println!("IR: {:?}", ir);
        return;
    }

    // TODO: Implement hierarchical tree format for IR
    println!("IR printing not yet implemented");
}
