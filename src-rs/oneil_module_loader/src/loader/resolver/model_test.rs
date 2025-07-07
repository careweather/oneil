use std::collections::{HashMap, HashSet};

use oneil_ast as ast;
use oneil_module::{
    reference::Identifier,
    test::{ModelTest, SubmodelTest, SubmodelTestInputs, TestIndex},
};

use crate::{
    error::{self, ModelTestResolutionError, SubmodelTestInputResolutionError},
    loader::resolver::{
        ModuleInfo, ParameterInfo, SubmodelInfo, expr::resolve_expr,
        trace_level::resolve_trace_level,
    },
};

pub fn resolve_model_tests(
    tests: Vec<ast::Test>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> (
    HashMap<TestIndex, ModelTest>,
    HashMap<TestIndex, Vec<ModelTestResolutionError>>,
) {
    let tests = tests.into_iter().enumerate().map(|(test_index, test)| {
        let test_index = TestIndex::new(test_index);

        let trace_level = resolve_trace_level(&test.trace_level);

        // TODO: verify that there are no duplicate inputs
        let inputs = test
            .inputs
            .into_iter()
            .map(|input| Identifier::new(input))
            .collect();

        let local_variables = &inputs;

        let test_expr = resolve_expr(
            &test.expr,
            local_variables,
            defined_parameters_info,
            submodel_info,
            module_info,
        )
        .map_err(|errors| (test_index.clone(), error::convert_errors(errors)))?;

        Ok((test_index, ModelTest::new(trace_level, inputs, test_expr)))
    });

    error::split_ok_and_errors(tests)
}

pub fn resolve_submodel_tests(
    submodel_tests: Vec<(Identifier, Vec<ast::declaration::ModelInput>)>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> (
    Vec<SubmodelTest>,
    HashMap<Identifier, Vec<SubmodelTestInputResolutionError>>,
) {
    let submodel_tests = submodel_tests.into_iter().map(|(submodel_name, inputs)| {
        // TODO: verify that there are no duplicate inputs
        let inputs: Vec<_> = inputs
            .into_iter()
            .map(|input| {
                let identifier = Identifier::new(input.name);
                let value = resolve_expr(
                    &input.value,
                    &HashSet::new(),
                    defined_parameters_info,
                    submodel_info,
                    module_info,
                )?;

                Ok((identifier, value))
            })
            .collect();

        let inputs = error::combine_error_list(inputs)
            .map_err(|errors| (submodel_name.clone(), error::convert_errors(errors)))?;
        let inputs = HashMap::from_iter(inputs);
        let inputs = SubmodelTestInputs::new(inputs);

        Ok(SubmodelTest::new(submodel_name, inputs))
    });

    error::split_ok_and_errors(submodel_tests)
}
