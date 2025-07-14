use oneil_ir::{model::Model, reference::Identifier};
use oneil_unit::Unit;

// For some expressions, specifically external function calls, we can't infer
// the units.  In this case, we return `None` to indicate that the units are not
// known. This is treated as an `any` unit, making this unit inference unsound.
//
// For this reason, we unfortunately also have to include unit information at
// runtime. Maybe later, we will find a way to annotate imported function types
// so that we can improve the space efficiency of evaluation.
