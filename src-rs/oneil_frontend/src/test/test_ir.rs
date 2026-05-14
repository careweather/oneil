use std::path::PathBuf;

use indexmap::IndexMap;

use oneil_ir as ir;
use oneil_shared::{
    labels::ParameterLabel,
    paths::{ModelPath, PythonPath},
    span::Span,
    symbols::{ParameterName, ReferenceName, SubmodelName, TestIndex},
};

use crate::instance::{
    InstancedModel,
    imports::{AliasImport, ReferenceImport, SubmodelImport},
};

/// Generates a span for testing purposes.
///
/// The span is intentionally random in order to discourage any
/// use of the spans for testing.
fn unimportant_span() -> Span {
    Span::random_span()
}

/// Generates a model path for testing purposes.
///
/// The path is intentionally random in order to discourage any
/// use of the path for testing.
fn unimportant_model_path() -> ModelPath {
    let path = PathBuf::from("unimportant.on");
    ModelPath::from_path_with_ext(&path)
}

// SIMPLE CONSTRUCTORS

pub fn expr_literal_number(value: f64) -> ir::Expr {
    let span = unimportant_span();
    ir::Expr::literal(span, ir::Literal::number(value))
}

pub fn empty_model() -> InstancedModel {
    InstancedModel::new(
        unimportant_model_path(),
        IndexMap::new(),
        IndexMap::new(),
        IndexMap::new(),
        IndexMap::new(),
        IndexMap::new(),
        IndexMap::new(),
        None,
    )
}

// BUILDERS

pub struct ModelBuilder {
    python_imports: IndexMap<PythonPath, ir::PythonImport>,
    submodels: IndexMap<ReferenceName, SubmodelImport>,
    references: IndexMap<ReferenceName, ReferenceImport>,
    aliases: IndexMap<ReferenceName, AliasImport>,
    parameters: IndexMap<ParameterName, ir::Parameter>,
    tests: IndexMap<TestIndex, ir::Test>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            python_imports: IndexMap::new(),
            submodels: IndexMap::new(),
            references: IndexMap::new(),
            aliases: IndexMap::new(),
            parameters: IndexMap::new(),
            tests: IndexMap::new(),
        }
    }

    pub fn with_submodel(mut self, submodel_name: &str, submodel_path: &ModelPath) -> Self {
        let span = unimportant_span();

        let reference_name = ReferenceName::new(submodel_name.to_string());
        let source_name = SubmodelName::new(submodel_name.to_string());
        let submodel_import = SubmodelImport::stub(source_name, span, submodel_path.clone());

        self.submodels.insert(reference_name, submodel_import);
        self
    }

    pub fn build(self) -> InstancedModel {
        InstancedModel::new(
            unimportant_model_path(),
            self.python_imports,
            self.submodels,
            self.references,
            self.aliases,
            self.parameters,
            self.tests,
            None,
        )
    }
}

pub struct ParameterBuilder {
    name: Option<ParameterName>,
    name_span: Option<Span>,
    span: Option<Span>,
    value: Option<ir::ParameterValue>,
    limits: Option<ir::Limits>,
    is_performance: bool,
    trace_level: ir::TraceLevel,
}

impl ParameterBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            name_span: None,
            span: None,
            value: None,
            limits: None,
            is_performance: false,
            trace_level: ir::TraceLevel::None,
        }
    }

    pub fn with_name_str(mut self, name: &str) -> Self {
        let name = ParameterName::from(name);
        self.name = Some(name);
        let span = unimportant_span();
        self.name_span = Some(span.clone());
        self.span = Some(span);
        self
    }

    pub fn with_simple_number_value(mut self, value: f64) -> Self {
        let expr = expr_literal_number(value);
        let value = ir::ParameterValue::simple(expr, None);
        self.value = Some(value);
        self
    }

    pub fn build(self) -> ir::Parameter {
        let name = self.name.expect("name must be set");
        let name_span = self.name_span.unwrap_or_else(unimportant_span);
        let span = self.span.unwrap_or_else(unimportant_span);
        let label = ParameterLabel::from(name.as_str());
        let value = self.value.expect("value must be set");
        let limits = self.limits.unwrap_or_default();

        ir::Parameter::new(
            ir::Dependencies::new(),
            name,
            name_span,
            span,
            label,
            None,
            None,
            value,
            limits,
            self.is_performance,
            self.trace_level,
            None,
        )
    }
}
