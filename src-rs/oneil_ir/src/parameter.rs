//! Parameter definitions and management for Oneil model IR.

use indexmap::IndexMap;

use oneil_shared::{
    RelativePath,
    labels::{ParameterLabel, RenderName, SectionLabel},
    paths::{DesignPath, ModelPath},
    span::Span,
    symbols::{BuiltinValueName, ParameterName, ReferenceName},
};

use crate::{debug_info::TraceLevel, expr::Expr, note::Note, unit::CompositeUnit};

/// Records the `apply <design> to <ref>` statement that produced a
/// design contribution.
///
/// Used by the runtime error reporter to surface a generic "applied
/// design produced invalid contributions" diagnostic at the apply
/// statement's span whenever the contribution it carried in fails
/// downstream (unit mismatch, missing target, post-composition cycle).
/// This lets a model file that applies a faulty design get its
/// `apply X to Y` line marked even though the precise failure lives
/// inside the design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignApplication {
    /// Span of the `apply <design> to <ref>` statement.
    pub apply_span: Span,
    /// File the `apply` statement lives in (a model `.on` file for an
    /// own-apply, a design `.one` file for a design's nested apply).
    pub applied_in: ModelPath,
    /// Path of the design file that this `apply` brought in.
    pub design_path: DesignPath,
}

/// Records which design last set a parameter's value on a given
/// instance.
///
/// Written by the instance-graph build pass in `apply_overlay_at_host`
/// (`oneil_frontend::instance::graph`) whenever a design override or
/// addition lands on a parameter. Read by:
/// - the runtime error reporter (`oneil_runtime::runtime::error`) to
///   attribute downstream failures back to the originating `apply`,
/// - the evaluator to push the anchor's scope before forcing the
///   overlay's RHS so variable references resolve in the design's
///   lexical scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignProvenance {
    /// Path of the design file (`.one`) that wrote this value.
    ///
    /// Always present — even when the design was supplied directly on the
    /// CLI (e.g. `oneil eval design.one`) rather than via an `apply`
    /// declaration, in which case [`applied_via`](Self::applied_via) is `None`.
    pub design_path: ModelPath,
    /// `true` when the design *added* this parameter to the host model
    /// (it did not exist on the base model); `false` when the design
    /// *overrode* a value that already existed on the host.
    pub is_addition: bool,
    /// Span of the `param = value` assignment line in the design file
    /// that wrote this value.
    pub assignment_span: Span,
    /// Path from the host instance to the anchor instance whose
    /// lexical scope owns the contributing expression.
    ///
    /// Eval resolves this against the host's absolute key at force
    /// time to find the anchor and push its scope so variable
    /// references in the overlay RHS bind to the design's own scope.
    pub anchor_path: RelativePath,
    /// The `apply X to Y` statement that brought this contribution
    /// in, when one exists.
    ///
    /// `None` for synthetic applies with no source span — currently
    /// only the "design as direct CLI root" path, where the design's
    /// own contributions overlay the target with no `apply` statement
    /// to attribute to. `Some(_)` in every other case.
    ///
    /// The build pass never composes multi-hop chains here: a child
    /// unit's contributions are baked into its cached graph with
    /// their original 1-hop `applied_via` and reused as-is when the
    /// unit is included as a submodel. Cross-cache propagation is
    /// handled separately by the "submodel `<alias>` has errors"
    /// notification in `emit_submodel_import_notifications`.
    pub applied_via: Option<DesignApplication>,
}

/// Represents a single parameter in an Oneil model.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    dependencies: Dependencies,
    name: ParameterName,
    name_span: Span,
    span: Span,
    label: ParameterLabel,
    /// Optional LaTeX render-name written as `{...}` after the `:` in source.
    /// When `Some`, the frontend uses this raw LaTeX string instead of deriving
    /// a symbol from the identifier.
    render_name: Option<RenderName>,
    section_label: Option<SectionLabel>,
    value: ParameterValue,
    limits: Limits,
    is_performance: bool,
    trace_level: TraceLevel,
    note: Option<Note>,
    /// Set when a design last applied a value to this parameter.
    /// `None` for parameters whose value comes from the model file itself.
    design_provenance: Option<DesignProvenance>,
}

impl Parameter {
    /// Creates a new parameter with the specified properties.
    ///
    /// `design_provenance` starts as `None`; use
    /// [`with_design_provenance`](Self::with_design_provenance) to attach
    /// provenance when a design applies a value to this parameter.
    #[expect(clippy::too_many_arguments, reason = "this is a constructor")]
    #[must_use]
    pub const fn new(
        dependencies: Dependencies,
        name: ParameterName,
        name_span: Span,
        span: Span,
        label: ParameterLabel,
        render_name: Option<RenderName>,
        section_label: Option<SectionLabel>,
        value: ParameterValue,
        limits: Limits,
        is_performance: bool,
        trace_level: TraceLevel,
        note: Option<Note>,
    ) -> Self {
        Self {
            dependencies,
            name,
            name_span,
            span,
            label,
            render_name,
            section_label,
            value,
            limits,
            is_performance,
            trace_level,
            note,
            design_provenance: None,
        }
    }

    /// Returns a copy of this parameter with the given design provenance attached.
    ///
    /// Called by `apply_overlay_at_host` in the instance-graph build
    /// pass for both addition and override contributions, immediately
    /// before the new value is installed via [`Self::value_mut`].
    #[must_use]
    pub fn with_design_provenance(mut self, provenance: DesignProvenance) -> Self {
        self.design_provenance = Some(provenance);
        self
    }

    /// Returns the design provenance for this parameter, if a design
    /// applied its value.
    ///
    /// `None` for parameters whose value comes straight from the
    /// model file. Read by the runtime error reporter to attribute
    /// downstream failures back to the originating `apply` and by
    /// the evaluator to set up the design's anchor scope before
    /// forcing the parameter.
    #[must_use]
    pub const fn design_provenance(&self) -> Option<&DesignProvenance> {
        self.design_provenance.as_ref()
    }

    /// Returns a reference to the set of parameter dependencies.
    #[must_use]
    pub const fn dependencies(&self) -> &Dependencies {
        &self.dependencies
    }

    /// Returns the name of this parameter.
    #[must_use]
    pub const fn name(&self) -> &ParameterName {
        &self.name
    }

    /// Returns the span of this parameter's identifier.
    #[must_use]
    pub const fn name_span(&self) -> &Span {
        &self.name_span
    }

    /// Returns the span covering the entire parameter definition.
    #[must_use]
    pub const fn span(&self) -> &Span {
        &self.span
    }

    /// Returns the label of this parameter.
    #[must_use]
    pub const fn label(&self) -> &ParameterLabel {
        &self.label
    }

    /// Returns the optional LaTeX render-name of this parameter.
    #[must_use]
    pub const fn render_name(&self) -> Option<&RenderName> {
        self.render_name.as_ref()
    }

    /// Returns the value of this parameter.
    #[must_use]
    pub const fn value(&self) -> &ParameterValue {
        &self.value
    }

    /// Returns a mutable reference to the value of this parameter.
    ///
    /// Used by the instance-graph build pass for two purposes:
    /// - `apply_overlay_at_host` overwrites the value when a design
    ///   override targets this parameter,
    /// - `link_value` walks the value's expression tree to classify
    ///   bare names against the host instance's binding scope
    ///   (turning `Parameter("g")` into `Builtin("g")` when `g`
    ///   isn't a parameter in scope but is a builtin, and similar).
    ///   Variable resolution to a target instance is deferred to
    ///   eval, so this walk only changes which `Variable::*` variant
    ///   carries each name — it does not attach instance keys.
    pub const fn value_mut(&mut self) -> &mut ParameterValue {
        &mut self.value
    }

    /// Returns the limits of this parameter.
    #[must_use]
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }

    /// Returns whether this parameter is a performance parameter.
    #[must_use]
    pub const fn is_performance(&self) -> bool {
        self.is_performance
    }

    /// Returns the trace level of this parameter.
    #[must_use]
    pub const fn trace_level(&self) -> TraceLevel {
        self.trace_level
    }

    /// Returns the section label for this parameter, if it was declared under a section.
    #[must_use]
    pub const fn section_label(&self) -> Option<&SectionLabel> {
        self.section_label.as_ref()
    }

    /// Returns the optional documentation note for this parameter.
    #[must_use]
    pub const fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }

    /// Replaces the documentation note.
    ///
    /// Called by the instance-graph build pass when a design override
    /// carries a design-specific note that should supersede the base
    /// model's note for this parameter.
    pub fn set_note(&mut self, note: Note) {
        self.note = Some(note);
    }

    /// Replaces the human-readable label.
    ///
    /// Called by the instance-graph build pass when a design override
    /// supplies an alternative label for the parameter.
    pub fn set_label(&mut self, label: ParameterLabel) {
        self.label = label;
    }

    /// Replaces the LaTeX render-name.
    ///
    /// Called by the instance-graph build pass when a design override
    /// supplies an alternative render-name for the parameter.
    pub fn set_render_name(&mut self, render_name: RenderName) {
        self.render_name = Some(render_name);
    }

    /// Replaces the section label.
    ///
    /// Called by the instance-graph build pass when a design line declares
    /// the parameter under a section header.
    pub fn set_section_label(&mut self, section_label: Option<SectionLabel>) {
        self.section_label = section_label;
    }

    /// Replaces the parameter limits.
    ///
    /// Called by the instance-graph build pass when a design override
    /// adjusts the target parameter's limits.
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// Mutable view of the dependency map.
    pub const fn dependencies_mut(&mut self) -> &mut Dependencies {
        &mut self.dependencies
    }
}

/// Names a parameter's RHS depends on, partitioned by binding kind.
///
/// Populated during file-time resolution by
/// `oneil_frontend::resolver::resolve_expr` as it walks the
/// expression tree. The frontend records names without resolving them
/// to a specific instance — `external` is keyed by
/// `(reference_name, parameter_name)` only; the model that
/// `reference_name` points to is determined later by walking the live
/// instance graph at validation / eval time.
///
/// Read by `oneil_analysis::dependency` to build the per-instance
/// dependency graph that feeds SCC detection
/// (`validate_instance_graph`) and by the evaluator to know what to
/// force first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependencies {
    /// Names that bind to a builtin value (constants like `pi`,
    /// builtin functions like `sin`).
    builtin: IndexMap<BuiltinValueName, Span>,
    /// Names that bind to parameters on the *same* instance.
    parameter: IndexMap<ParameterName, Span>,
    /// `parameter.reference` accesses, keyed by
    /// (`reference_name`, `parameter_name`). The reference's target
    /// instance is resolved lazily from the live instance graph
    /// rather than cached here, so an overlay or addition that
    /// changes a reference's binding doesn't require rebuilding
    /// dependencies.
    external: IndexMap<(ReferenceName, ParameterName), Span>,
}

impl Dependencies {
    /// Creates new dependencies.
    #[must_use]
    pub fn new() -> Self {
        Self {
            builtin: IndexMap::new(),
            parameter: IndexMap::new(),
            external: IndexMap::new(),
        }
    }

    /// Returns the names that resolve to builtin values, with the
    /// span of each occurrence.
    #[must_use]
    pub const fn builtin(&self) -> &IndexMap<BuiltinValueName, Span> {
        &self.builtin
    }

    /// Returns the names that resolve to parameters on the same
    /// instance, with the span of each occurrence.
    #[must_use]
    pub const fn parameter(&self) -> &IndexMap<ParameterName, Span> {
        &self.parameter
    }

    /// Returns the `parameter.reference` accesses on this RHS, keyed
    /// by (reference name, parameter name).
    #[must_use]
    pub const fn external(&self) -> &IndexMap<(ReferenceName, ParameterName), Span> {
        &self.external
    }

    /// Records a builtin-name occurrence. Called by `resolve_expr`
    /// when a bare identifier matches a registered builtin.
    pub fn insert_builtin(&mut self, ident: BuiltinValueName, span: Span) {
        self.builtin.insert(ident, span);
    }

    /// Records a same-instance parameter-name occurrence. Called by
    /// `resolve_expr` when a bare identifier matches a parameter in
    /// the active model's scope.
    pub fn insert_parameter(&mut self, parameter_name: ParameterName, span: Span) {
        self.parameter.insert(parameter_name, span);
    }

    /// Records a `parameter.reference` access. Called by
    /// `resolve_expr` for every dotted access in the RHS.
    pub fn insert_external(
        &mut self,
        reference_name: ReferenceName,
        parameter_name: ParameterName,
        full_span: Span,
    ) {
        self.external
            .insert((reference_name, parameter_name), full_span);
    }

    /// Merges another `Dependencies` into this one.
    pub fn extend(&mut self, other: Self) {
        self.builtin.extend(other.builtin);
        self.parameter.extend(other.parameter);
        self.external.extend(other.external);
    }
}

impl Default for Dependencies {
    fn default() -> Self {
        Self::new()
    }
}

/// The value of a parameter, which can be either a simple expression or a piecewise function.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ParameterValue {
    /// A simple expression with an optional unit.
    Simple(Box<Expr>, Option<CompositeUnit>),
    /// A piecewise function with multiple expressions and conditions.
    Piecewise(Vec<PiecewiseExpr>, Option<CompositeUnit>),
}

impl ParameterValue {
    /// Creates a simple parameter value from an expression and optional unit.
    #[must_use]
    pub fn simple(expr: Expr, unit: Option<CompositeUnit>) -> Self {
        Self::Simple(Box::new(expr), unit)
    }

    /// Creates a piecewise parameter value from a list of expressions and conditions.
    #[must_use]
    pub const fn piecewise(exprs: Vec<PiecewiseExpr>, unit: Option<CompositeUnit>) -> Self {
        Self::Piecewise(exprs, unit)
    }
}

/// A single expression in a piecewise function with its associated condition.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PiecewiseExpr {
    expr: Expr,
    if_expr: Expr,
}

impl PiecewiseExpr {
    /// Creates a new piecewise expression with a value and condition.
    #[must_use]
    pub const fn new(expr: Expr, if_expr: Expr) -> Self {
        Self { expr, if_expr }
    }

    /// Returns the expression value.
    #[must_use]
    pub const fn expr(&self) -> &Expr {
        &self.expr
    }

    /// Returns the condition expression.
    #[must_use]
    pub const fn if_expr(&self) -> &Expr {
        &self.if_expr
    }

    /// Returns a mutable reference to the value expression.
    ///
    /// Walked by the build pass's `link_value` to classify bare names
    /// against the surrounding instance's binding scope. See
    /// [`Parameter::value_mut`] for the full classification contract.
    pub const fn expr_mut(&mut self) -> &mut Expr {
        &mut self.expr
    }

    /// Returns a mutable reference to the condition expression.
    ///
    /// Walked by the build pass's `link_value` for the same name
    /// classification step that handles the value side.
    pub const fn if_expr_mut(&mut self) -> &mut Expr {
        &mut self.if_expr
    }
}

/// Constraints on valid parameter values.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Limits {
    /// No constraints on parameter values.
    #[default]
    Default,
    /// Continuous range with minimum and maximum values.
    Continuous {
        /// The minimum allowed value expression.
        min: Box<Expr>,
        /// The maximum allowed value expression.
        max: Box<Expr>,
        /// The span of the expression representing the limit.
        limit_expr_span: Span,
    },
    /// Discrete set of allowed values.
    Discrete {
        /// Vector of expressions representing allowed values.
        values: Vec<Expr>,
        /// The span of the expression representing the limit.
        limit_expr_span: Span,
    },
}

impl Limits {
    /// Creates default limits (no constraints).
    #[must_use]
    pub const fn default() -> Self {
        Self::Default
    }

    /// Creates continuous limits with minimum and maximum expressions.
    #[must_use]
    pub fn continuous(min: Expr, max: Expr, limit_expr_span: Span) -> Self {
        Self::Continuous {
            min: Box::new(min),
            max: Box::new(max),
            limit_expr_span,
        }
    }

    /// Creates discrete limits with a set of allowed values.
    #[must_use]
    pub const fn discrete(values: Vec<Expr>, limit_expr_span: Span) -> Self {
        Self::Discrete {
            values,
            limit_expr_span,
        }
    }
}
