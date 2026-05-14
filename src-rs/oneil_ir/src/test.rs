//! Testing for Oneil model IR.

use oneil_shared::{labels::SectionLabel, span::Span};

use crate::{Dependencies, DesignProvenance, Note, debug_info::TraceLevel, expr::Expr};

/// A test within a model.
#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    span: Span,
    trace_level: TraceLevel,
    expr: Expr,
    dependencies: Dependencies,
    section_label: Option<SectionLabel>,
    /// Optional documentation note attached to the test.
    note: Option<Note>,
    /// Set when this test was added by a design file.
    /// `None` for tests defined in the model file itself.
    design_provenance: Option<DesignProvenance>,
}

impl Test {
    /// Creates a new test with the specified properties.
    ///
    /// `design_provenance` starts as `None`; use
    /// [`with_design_provenance`](Self::with_design_provenance) to attach
    /// provenance when a design adds this test to a target model.
    #[must_use]
    pub const fn new(
        span: Span,
        trace_level: TraceLevel,
        expr: Expr,
        dependencies: Dependencies,
        section_label: Option<SectionLabel>,
        note: Option<Note>,
    ) -> Self {
        Self {
            span,
            trace_level,
            expr,
            dependencies,
            section_label,
            note,
            design_provenance: None,
        }
    }

    /// Returns this test with design provenance attached.
    ///
    /// Called when a design file adds a test to a target model.
    #[must_use]
    pub fn with_design_provenance(mut self, provenance: DesignProvenance) -> Self {
        self.design_provenance = Some(provenance);
        self
    }

    /// Returns the span of the entire test definition.
    #[must_use]
    pub const fn span(&self) -> &Span {
        &self.span
    }

    /// Returns the trace level for this test.
    #[must_use]
    pub const fn trace_level(&self) -> TraceLevel {
        self.trace_level
    }

    /// Returns the test expression that defines the expected behavior.
    #[must_use]
    pub const fn expr(&self) -> &Expr {
        &self.expr
    }

    /// Returns a mutable reference to the test expression.
    pub const fn expr_mut(&mut self) -> &mut Expr {
        &mut self.expr
    }

    /// Returns the dependencies of this test.
    #[must_use]
    pub const fn dependencies(&self) -> &Dependencies {
        &self.dependencies
    }

    /// Returns the section label for this test, if any.
    #[must_use]
    pub const fn section_label(&self) -> Option<&SectionLabel> {
        self.section_label.as_ref()
    }

    /// Returns the optional documentation note for this test.
    #[must_use]
    pub const fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }

    /// Returns the design provenance for this test, if it was added by a design.
    ///
    /// `None` for tests defined in the model file itself.
    #[must_use]
    pub const fn design_provenance(&self) -> Option<&DesignProvenance> {
        self.design_provenance.as_ref()
    }
}
