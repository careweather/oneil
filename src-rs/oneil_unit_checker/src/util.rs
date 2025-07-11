use std::ops::{Deref, DerefMut};

use oneil_shared::{ModelMap, ModelMapBuilder};

#[derive(Debug, Clone, PartialEq)]
pub struct UnitMap(ModelMap<oneil_unit::Unit>);

impl Deref for UnitMap {
    type Target = ModelMap<oneil_unit::Unit>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitMapBuilder(ModelMapBuilder<oneil_unit::Unit, ()>);

impl UnitMapBuilder {
    pub fn new() -> Self {
        Self(ModelMapBuilder::new())
    }
}

impl Deref for UnitMapBuilder {
    type Target = oneil_shared::ModelMapBuilder<oneil_unit::Unit, ()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnitMapBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryInto<UnitMap> for UnitMapBuilder {
    type Error = (UnitMap, ModelMap<()>);

    fn try_into(self) -> Result<UnitMap, Self::Error> {
        self.0
            .try_into()
            .map(UnitMap)
            .map_err(|(map, errors)| (UnitMap(map), errors))
    }
}
