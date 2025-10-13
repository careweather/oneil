use std::collections::{HashMap, HashSet};

use oneil_ir::{self as ir, IrSpan};

fn unimportant_span() -> IrSpan {
    use rand::Rng;
    let mut rng = rand::rng();

    let start = usize::from(rng.random::<u16>());
    let length = usize::from(rng.random::<u16>());
    IrSpan::new(start, length)
}

// SIMPLE CONSTRUCTORS

pub fn reference_name_with_span(reference_name: &str) -> ir::ReferenceNameWithSpan {
    let reference_name = ir::ReferenceName::new(reference_name.to_string());
    ir::ReferenceNameWithSpan::new(reference_name, unimportant_span())
}

pub fn expr_literal_number(value: f64) -> ir::ExprWithSpan {
    let expr = ir::Expr::literal(ir::Literal::number(value));
    ir::WithSpan::new(expr, unimportant_span())
}

pub fn empty_model() -> ir::Model {
    ir::Model::new(
        HashMap::new(),
        ir::SubmodelMap::new(HashMap::new()),
        ir::ReferenceMap::new(HashMap::new()),
        ir::ParameterCollection::new(HashMap::new()),
        HashMap::new(),
    )
}

// BUILDERS

pub struct ModelBuilder {
    python_imports: HashMap<ir::PythonPath, IrSpan>,
    submodels: HashMap<ir::SubmodelName, ir::SubmodelImport>,
    references: HashMap<ir::ReferenceName, ir::ReferenceImport>,
    parameters: HashMap<ir::Identifier, ir::Parameter>,
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
        let submodel_name_with_span = ir::WithSpan::new(submodel_name.clone(), unimportant_span());
        let model_path = ir::ModelPath::new(submodel_path);

        let submodel_import = ir::SubmodelImport::new(submodel_name_with_span, model_path);

        self.submodels.insert(submodel_name, submodel_import);
        self
    }

    pub fn with_literal_number_parameter(mut self, ident: &str, value: f64) -> Self {
        let parameter = ParameterBuilder::new()
            .with_ident_str(ident)
            .with_simple_number_value(value)
            .build();

        self.parameters
            .insert(ir::Identifier::new(ident), parameter);

        self
    }

    pub fn build(self) -> ir::Model {
        let submodel_map = ir::SubmodelMap::new(self.submodels);
        let reference_map = ir::ReferenceMap::new(self.references);
        let parameter_collection = ir::ParameterCollection::new(self.parameters);
        ir::Model::new(
            self.python_imports,
            submodel_map,
            reference_map,
            parameter_collection,
            self.tests,
        )
    }
}

pub struct ParameterBuilder {
    dependencies: HashSet<ir::Identifier>,
    ident: Option<ir::IdentifierWithSpan>,
    value: Option<ir::ParameterValue>,
    limits: Option<ir::Limits>,
    is_performance: bool,
    trace_level: ir::TraceLevel,
}

impl ParameterBuilder {
    pub fn new() -> Self {
        Self {
            dependencies: HashSet::new(),
            ident: None,
            value: None,
            limits: None,
            is_performance: false,
            trace_level: ir::TraceLevel::None,
        }
    }

    pub fn with_ident_str(mut self, ident: &str) -> Self {
        let ident_with_span =
            ir::IdentifierWithSpan::new(ir::Identifier::new(ident), unimportant_span());
        self.ident = Some(ident_with_span);

        self
    }

    pub fn with_simple_number_value(mut self, value: f64) -> Self {
        let expr = expr_literal_number(value);
        let value = ir::ParameterValue::simple(expr, None);
        self.value = Some(value);

        self
    }

    pub fn build(self) -> ir::Parameter {
        let ident = self.ident.expect("ident must be set");
        let value = self.value.expect("value must be set");
        let limits = self.limits.unwrap_or_default();
        let is_performance = self.is_performance;
        let trace_level = self.trace_level;

        ir::Parameter::new(
            self.dependencies,
            ident,
            value,
            limits,
            is_performance,
            trace_level,
        )
    }
}
