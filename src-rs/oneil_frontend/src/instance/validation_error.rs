//! Errors emitted by the post-build validation pass over an
//! [`InstanceGraph`](super::InstanceGraph).
//!
//! Validation runs once the graph is fully built and applies all
//! cross-instance checks: undefined parameter / reference names, `p.r`
//! existence, and parameter dependency cycles.
//!
//! Errors carry a `host_path: InstancePath` from the graph's root to
//! the affected instance. Cycle errors carry the cycle as a list of
//! `(host_path, parameter_name)` pairs.

use oneil_shared::{
    InstancePath,
    error::{AsOneilDiagnostic, Context, DiagnosticKind, ErrorLocation},
    paths::ModelPath,
    span::Span,
    symbols::{ParameterName, ReferenceName, TestIndex},
};

/// One issue detected by the post-build validation pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceValidationError {
    /// Path from the graph's root to the host instance whose
    /// parameter or test contains the invalid reference.
    pub host_path: InstancePath,
    /// Where in the host the error was found (parameter or test).
    pub host_location: HostLocation,
    /// What went wrong.
    pub kind: InstanceValidationErrorKind,
}

/// Where in an [`InstancedModel`] a validation error was found.
///
/// [`InstancedModel`]: super::InstancedModel
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HostLocation {
    /// In the right-hand side of a parameter declaration.
    Parameter(ParameterName),
    /// In a test expression.
    Test(TestIndex),
}

/// Kinds of validation errors emitted by the post-build pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceValidationErrorKind {
    /// A bare parameter name in an expression could not be resolved to any
    /// parameter in the instance's own scope after all designs have been
    /// applied.
    UndefinedParameter {
        /// The name as written in the source.
        parameter_name: ParameterName,
        /// Span of the identifier (in the design file when `design_info` is
        /// `Some`, otherwise in the host model file).
        parameter_span: Span,
        /// Best fuzzy match among the instance's actual parameters, if any.
        best_match: Option<ParameterName>,
        /// When the containing parameter was set by a design, the design
        /// file path and the span of its `param = value` assignment line.
        design_info: Option<(ModelPath, Span)>,
    },
    /// A reference name in a `p.r` expression could not be resolved to any
    /// reference in the instance's scope after all designs have been applied.
    UndefinedReference {
        /// The reference name as written in the source (e.g. `r` in `p.r`).
        reference_name: ReferenceName,
        /// Span of the reference identifier (in the design file when
        /// `design_info` is `Some`, otherwise in the host model file).
        reference_span: Span,
        /// Best fuzzy match among the instance's references, if any.
        best_match: Option<ReferenceName>,
        /// When the containing parameter was set by a design, the design
        /// file path and the span of its `param = value` assignment line.
        design_info: Option<(ModelPath, Span)>,
    },
    /// `Variable::External { r }` where the parameter `p` in the source
    /// expression `p.r` is not defined on the target instance after all
    /// designs have been applied. The source spelling is `p.r` (subscript
    /// style); `r` is stored in the IR and `p` is resolved at validation time.
    UndefinedReferenceParameter {
        /// The reference name as written in the source (e.g. `r` in `p.r`).
        reference_name: ReferenceName,
        /// Span of the reference identifier (in the design file when
        /// `design_info` is `Some`, otherwise in the host model file).
        reference_span: Span,
        /// The parameter name as written in the source (e.g. `p` in `p.r`).
        parameter_name: ParameterName,
        /// Span of the parameter identifier (in the design file when
        /// `design_info` is `Some`, otherwise in the host model file).
        parameter_span: Span,
        /// The model the reference resolved to (where `p` was looked up).
        target_model: ModelPath,
        /// Best fuzzy match for `parameter_name` among the target's
        /// parameter names, if any.
        best_match: Option<ParameterName>,
        /// When the containing parameter was set by a design, the design
        /// file path and the span of its `param = value` assignment line.
        design_info: Option<(ModelPath, Span)>,
    },
    /// The host parameter participates in a parameter dependency cycle.
    ///
    /// SCC detection runs once on the fully-built graph (post-all-applies)
    /// so the cycle reflects every overlay contribution. One error is
    /// emitted per member of the strongly-connected component — each
    /// attached to its own host instance — with the `cycle` list rotated
    /// so the host parameter is the first entry.
    ParameterCycle {
        /// The host parameter's name (also stored as part of `cycle[0]`).
        parameter_name: ParameterName,
        /// Span of the host parameter's identifier.
        parameter_span: Span,
        /// When the cycle was introduced by a design, the design file path
        /// and the span of the `param = value` assignment that created the
        /// problematic dependency.
        design_info: Option<(ModelPath, Span)>,
        /// The cycle in dependency order, starting at the host parameter
        /// and looping back to itself implicitly. Length `>= 1`; a length
        /// of 1 indicates a self-referential parameter.
        cycle: Vec<CycleMember>,
    },
    /// The host reference has errors internally.
    ReferenceHasError {
        /// The reference name as written in the source (e.g. `r` in `p.r`).
        reference_name: ReferenceName,
        /// Span of the reference identifier (in the design file when
        /// `design_info` is `Some`, otherwise in the host model file).
        reference_span: Span,
        /// When the containing parameter was set by a design, the design
        /// file path and the span of its `param = value` assignment line.
        design_info: Option<(ModelPath, Span)>,
    },
}

/// One link in a parameter-dependency cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleMember {
    /// Path from the graph's root to the instance owning the parameter.
    pub host_path: InstancePath,
    /// Model file the instance was lowered from. Used for cross-instance
    /// disambiguation in cycle messages.
    pub model: ModelPath,
    /// The parameter's name.
    pub parameter_name: ParameterName,
}

impl InstanceValidationError {
    /// Returns the kind of validation error.
    #[must_use]
    pub const fn kind(&self) -> &InstanceValidationErrorKind {
        &self.kind
    }

    /// Returns the source span the error attaches to. Used by the runtime's
    /// error rendering pipeline to compute line/column information.
    #[must_use]
    pub fn primary_span(&self) -> Span {
        match &self.kind {
            InstanceValidationErrorKind::UndefinedParameter { parameter_span, .. }
            | InstanceValidationErrorKind::UndefinedReferenceParameter { parameter_span, .. }
            | InstanceValidationErrorKind::ParameterCycle { parameter_span, .. } => {
                parameter_span.clone()
            }
            InstanceValidationErrorKind::UndefinedReference { reference_span, .. }
            | InstanceValidationErrorKind::ReferenceHasError { reference_span, .. } => {
                reference_span.clone()
            }
        }
    }
}

impl AsOneilDiagnostic for InstanceValidationError {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn message(&self) -> String {
        match &self.kind {
            InstanceValidationErrorKind::UndefinedParameter {
                parameter_name,
                best_match,
                design_info,
                ..
            } => {
                let name = parameter_name.as_str();
                let suggestion = best_match
                    .as_ref()
                    .map(|m| format!(" (did you mean `{}`?)", m.as_str()))
                    .unwrap_or_default();
                if let Some((design_path, _)) = design_info {
                    let design = design_name(design_path);
                    format!("design `{design}` introduced undefined parameter `{name}`{suggestion}")
                } else {
                    format!("undefined parameter `{name}`{suggestion}")
                }
            }
            InstanceValidationErrorKind::UndefinedReference {
                reference_name,
                design_info,
                ..
            } => {
                let name = reference_name.as_str();
                if let Some((design_path, _)) = design_info {
                    let design = design_name(design_path);
                    format!("design `{design}` introduced undefined reference `{name}`")
                } else {
                    format!("undefined reference `{name}`")
                }
            }
            InstanceValidationErrorKind::UndefinedReferenceParameter {
                reference_name,
                parameter_name,
                target_model,
                design_info,
                ..
            } => {
                let reference = reference_name.as_str();
                let parameter = parameter_name.as_str();
                let target = target_model.as_path().display();
                if let Some((design_path, _)) = design_info {
                    let design = design_name(design_path);
                    format!(
                        "design `{design}` introduced undefined parameter: \
                         `{parameter}` is not defined on reference `{reference}` (model `{target}`)"
                    )
                } else {
                    format!(
                        "parameter `{parameter}` is not defined on reference `{reference}` (model `{target}`)"
                    )
                }
            }
            InstanceValidationErrorKind::ParameterCycle {
                cycle,
                parameter_name,
                design_info,
                ..
            } => {
                let chain = render_cycle_chain(cycle);
                let host = parameter_name.as_str();
                if let Some((design_path, _)) = design_info {
                    let d = design_name(design_path);
                    format!(
                        "design `{d}` introduced a circular dependency \
                         in parameters - {chain} -> {host}",
                    )
                } else {
                    format!("circular dependency detected in parameters - {chain} -> {host}")
                }
            }
            InstanceValidationErrorKind::ReferenceHasError { reference_name, .. } => {
                let reference = reference_name.as_str();
                format!("reference `{reference}` has errors")
            }
        }
    }

    fn diagnostic_location(&self, _source: &str) -> Option<ErrorLocation> {
        Some(ErrorLocation::from_span(&self.primary_span()))
    }

    fn context(&self) -> Vec<Context> {
        match &self.kind {
            InstanceValidationErrorKind::UndefinedParameter { best_match, .. } => best_match
                .as_ref()
                .map(ParameterName::as_str)
                .map(|m| vec![Context::Help(format!("did you mean `{m}`?"))])
                .unwrap_or_default(),
            InstanceValidationErrorKind::UndefinedReference { best_match, .. } => best_match
                .as_ref()
                .map(ReferenceName::as_str)
                .map(|m| vec![Context::Help(format!("did you mean `{m}`?"))])
                .unwrap_or_default(),
            InstanceValidationErrorKind::UndefinedReferenceParameter { best_match, .. } => {
                best_match
                    .as_ref()
                    .map(ParameterName::as_str)
                    .map(|m| vec![Context::Help(format!("did you mean `{m}`?"))])
                    .unwrap_or_default()
            }
            InstanceValidationErrorKind::ParameterCycle { .. }
            | InstanceValidationErrorKind::ReferenceHasError { .. } => Vec::new(),
        }
    }

    fn is_internal_diagnostic(&self) -> bool {
        matches!(
            self.kind,
            InstanceValidationErrorKind::ReferenceHasError { .. }
        )
    }
}

/// Returns the filename stem of a design path for use in error messages.
fn design_name(path: &ModelPath) -> &str {
    path.as_path()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("(unknown design)")
}

/// Renders a cycle as `a -> b -> r.c` style text. Cross-instance hops are
/// disambiguated with a leading `<model basename>::` prefix so two
/// parameters of the same name on different instances don't collapse into
/// each other in the message.
fn render_cycle_chain(cycle: &[CycleMember]) -> String {
    let mut out = String::new();
    let mut prev_path: Option<&InstancePath> = None;
    for member in cycle {
        if !out.is_empty() {
            out.push_str(" -> ");
        }
        let cross_instance = prev_path.is_some_and(|prev| prev != &member.host_path);
        if cross_instance
            && let Some(stem) = member.model.as_path().file_stem().and_then(|s| s.to_str())
        {
            out.push_str(stem);
            out.push_str("::");
        }
        out.push_str(member.parameter_name.as_str());
        prev_path = Some(&member.host_path);
    }
    out
}
