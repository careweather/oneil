//! Model map module for storing processed model traversal results.
//!
//! This module defines the data structures used to store the results of traversing and processing
//! models, including both successful results and errors, for all model components.

use std::collections::HashMap;

use oneil_ir::{
    reference::{Identifier, ModelPath, PythonPath},
    test::TestIndex,
};

/// A collection of processed data for a single model.
///
/// This struct holds the results of processing all components within a model:
/// python imports, submodels, parameters, tests, and submodel tests.
/// Each component type is stored in its own map for efficient lookup.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelMapEntry<PyT, SubmodelT, ParamT, TestT, SubmodelTestT> {
    python_imports_map: HashMap<PythonPath, PyT>,
    submodels_map: HashMap<Identifier, SubmodelT>,
    parameters_map: HashMap<Identifier, ParamT>,
    tests_map: HashMap<TestIndex, TestT>,
    submodel_tests_map: HashMap<Identifier, SubmodelTestT>,
}

impl<PyT, SubmodelT, ParamT, TestT, SubmodelTestT>
    ModelMapEntry<PyT, SubmodelT, ParamT, TestT, SubmodelTestT>
{
    /// Creates a new `ModelMapEntry` with the provided component maps.
    ///
    /// # Arguments
    ///
    /// * `python_imports_map` - Map of python import paths to their processed data
    /// * `submodels_map` - Map of submodel identifiers to their processed data
    /// * `parameters_map` - Map of parameter identifiers to their processed data
    /// * `tests_map` - Map of test indices to their processed data
    /// * `submodel_tests_map` - Map of submodel test identifiers to their processed data
    pub fn new(
        python_imports_map: HashMap<PythonPath, PyT>,
        submodels_map: HashMap<Identifier, SubmodelT>,
        parameters_map: HashMap<Identifier, ParamT>,
        tests_map: HashMap<TestIndex, TestT>,
        submodel_tests_map: HashMap<Identifier, SubmodelTestT>,
    ) -> Self {
        Self {
            python_imports_map,
            submodels_map,
            parameters_map,
            tests_map,
            submodel_tests_map,
        }
    }
}

/// A collection of processed data for multiple models.
///
/// This struct maps model paths to their corresponding `ModelMapEntry`,
/// allowing efficient lookup of processed data for any model in the collection.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelMap<PyT, SubmodelT, ParamT, TestT, SubmodelTestT> {
    map: HashMap<ModelPath, ModelMapEntry<PyT, SubmodelT, ParamT, TestT, SubmodelTestT>>,
}

impl<PyT, SubmodelT, ParamT, TestT, SubmodelTestT>
    ModelMap<PyT, SubmodelT, ParamT, TestT, SubmodelTestT>
{
    /// Creates a new `ModelMap` with the provided model entries.
    ///
    /// # Arguments
    ///
    /// * `map` - HashMap mapping model paths to their processed data entries
    pub fn new(
        map: HashMap<ModelPath, ModelMapEntry<PyT, SubmodelT, ParamT, TestT, SubmodelTestT>>,
    ) -> Self {
        Self { map }
    }
}
