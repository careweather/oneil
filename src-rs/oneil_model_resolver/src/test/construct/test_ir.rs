use std::collections::HashMap;

use oneil_ir as ir;
use oneil_shared::span::Span;

/// Generates a span for testing purposes
///
/// The span is intentionally random in order to discourage any
/// use of the spans for testing.
fn unimportant_span() -> Span {
    Span::random_span()
}

// SIMPLE CONSTRUCTORS

pub fn reference_name(reference_name: &str) -> ir::ReferenceName {
    ir::ReferenceName::new(reference_name.to_string())
}

pub fn expr_literal_number(value: f64) -> ir::Expr {
    let span = unimportant_span();
    ir::Expr::literal(span, ir::Literal::number(value))
}

pub fn empty_model() -> ir::Model {
    ir::Model::new(
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    )
}

// BUILDERS

pub struct ModelBuilder {
    python_imports: HashMap<ir::PythonPath, ir::PythonImport>,
    submodels: HashMap<ir::SubmodelName, ir::SubmodelImport>,
    references: HashMap<ir::ReferenceName, ir::ReferenceImport>,
    parameters: HashMap<ir::ParameterName, ir::Parameter>,
    tests: HashMap<ir::TestIndex, ir::Test>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            python_imports: HashMap::new(),
            submodels: HashMap::new(),
            references: HashMap::new(),
            parameters: HashMap::new(),
            tests: HashMap::new(),
        }
    }

    pub fn with_submodel(mut self, submodel_name: &str, submodel_path: &str) -> Self {
        let submodel_name = ir::SubmodelName::new(submodel_name.to_string());
        let submodel_name_span = unimportant_span();
        let model_path = ir::ModelPath::new(submodel_path);

        let submodel_import =
            ir::SubmodelImport::new(submodel_name.clone(), submodel_name_span, model_path);

        self.submodels.insert(submodel_name, submodel_import);
        self
    }

    pub fn with_literal_number_parameter(mut self, ident: &str, value: f64) -> Self {
        let parameter = ParameterBuilder::new()
            .with_name_str(ident)
            .with_simple_number_value(value)
            .build();

        self.parameters
            .insert(ir::ParameterName::new(ident.to_string()), parameter);

        self
    }

    pub fn build(self) -> ir::Model {
        ir::Model::new(
            self.python_imports,
            self.submodels,
            self.references,
            self.parameters,
            self.tests,
        )
    }
}

pub struct ParameterBuilder {
    name: Option<ir::ParameterName>,
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
        let name = ir::ParameterName::new(name.to_string());
        self.name = Some(name);
        let span = unimportant_span();
        self.name_span = Some(span);
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
        let label = ir::Label::new(name.as_str().to_string());
        let value = self.value.expect("value must be set");
        let limits = self.limits.unwrap_or_default();
        let is_performance = self.is_performance;
        let trace_level = self.trace_level;

        ir::Parameter::new(
            ir::Dependencies::new(),
            name,
            name_span,
            span,
            label,
            value,
            limits,
            is_performance,
            trace_level,
        )
    }
}
