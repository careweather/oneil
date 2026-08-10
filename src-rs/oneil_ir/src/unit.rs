//! Unit system for dimensional analysis in Oneil.

use oneil_output::{DimensionMap, DisplayUnit as ResolvedDisplayUnit, Unit as ResolvedUnit};
use oneil_shared::{
    serde::f64 as f64_serde,
    span::Span,
    symbols::{UnitBaseName, UnitName, UnitPrefix},
};

/// A composite unit composed of multiple base units.
///
/// Each composite unit carries its pre-resolved [`DimensionMap`] alongside
/// the AST-derived component breakdown. The dimension map is computed once
/// during lowering (when builtin unit definitions are in scope) so later
/// passes — design overlay validation, dimensional analysis — can compare
/// dimensions by data without re-evaluating the unit expression.
/// Serializes as its resolved display string (e.g. `"kg"`, `"m/s^2"`); the
/// `ts-bindings` override matches that wire shape rather than the internal
/// fields.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-bindings", ts(type = "string"))]
pub struct CompositeUnit {
    units: Vec<Unit>,
    display_unit: DisplayCompositeUnit,
    span: Span,
    /// Pre-resolved dimension map for this composite unit.
    ///
    /// Populated by the lowering pass once the builtin unit dictionary is
    /// consulted. Subsequent passes treat this as the authoritative
    /// dimensional signature of the unit.
    dimension: DimensionMap,
}

impl CompositeUnit {
    /// Creates a new composite unit from a vector of individual units.
    #[must_use]
    pub const fn new(
        units: Vec<Unit>,
        display_unit: DisplayCompositeUnit,
        span: Span,
        dimension: DimensionMap,
    ) -> Self {
        Self {
            units,
            display_unit,
            span,
            dimension,
        }
    }

    /// Returns a reference to the units in this composite unit.
    #[must_use]
    pub const fn units(&self) -> &[Unit] {
        self.units.as_slice()
    }

    /// Returns a reference to the display unit of this composite unit.
    #[must_use]
    pub const fn display_unit(&self) -> &DisplayCompositeUnit {
        &self.display_unit
    }

    /// Returns the span of this composite unit.
    #[must_use]
    pub const fn span(&self) -> &Span {
        &self.span
    }

    /// Returns the pre-resolved dimension map for this composite unit.
    #[must_use]
    pub const fn dimension(&self) -> &DimensionMap {
        &self.dimension
    }
}

/// A single unit with a name and exponent.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    span: Span,
    name: UnitName,
    name_span: Span,
    exponent: f64,
    exponent_span: Option<Span>,
    info: UnitInfo,
}

impl Unit {
    /// Creates a new unit with the specified name and exponent.
    #[must_use]
    pub const fn new(
        span: Span,
        name: UnitName,
        name_span: Span,
        exponent: f64,
        exponent_span: Option<Span>,
        info: UnitInfo,
    ) -> Self {
        Self {
            span,
            name,
            name_span,
            exponent,
            exponent_span,
            info,
        }
    }

    /// Returns the span of this unit.
    #[must_use]
    pub const fn span(&self) -> &Span {
        &self.span
    }

    /// Returns the name of this unit.
    #[must_use]
    pub const fn name(&self) -> &UnitName {
        &self.name
    }

    /// Returns the span of the name of this unit.
    #[must_use]
    pub const fn name_span(&self) -> &Span {
        &self.name_span
    }

    /// Returns the exponent of this unit.
    #[must_use]
    pub const fn exponent(&self) -> f64 {
        self.exponent
    }

    /// Returns the span of the exponent of this unit.
    #[must_use]
    pub const fn exponent_span(&self) -> Option<&Span> {
        self.exponent_span.as_ref()
    }

    /// Returns the unit info of this unit.
    #[must_use]
    pub const fn info(&self) -> &UnitInfo {
        &self.info
    }
}

impl serde::Serialize for CompositeUnit {
    /// Serializes a composite unit as its resolved display string.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let resolved = self.display_unit().to_resolved_display();
        serializer.serialize_str(&format!("{resolved}"))
    }
}

impl serde::Serialize for Unit {
    /// Serializes a unit as `{"name": "...", "exponent": f64}`.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Unit", 2)?;
        state.serialize_field("name", self.name().as_str())?;
        state.serialize_field("exponent", &self.exponent())?;
        state.end()
    }
}

/// Computes the [`DimensionMap`] of a list of [`Unit`]s by looking up each
/// base unit's dimension map and combining them according to the unit's
/// exponent.
///
/// The lookup function should return the resolved [`ResolvedUnit`] for a
/// builtin unit base name (typically forwarded to the runtime's builtin
/// table). Units whose base name cannot be resolved are treated as
/// dimensionless; missing-unit errors are surfaced separately during the
/// lowering pass and shouldn't be re-reported here.
///
/// dB units do not contribute to the dimension map (they're a logarithmic
/// scale, not a dimension), but their inner base unit, if any, does.
#[must_use]
pub fn compute_dimension_map<F>(units: &[Unit], mut lookup_unit: F) -> DimensionMap
where
    F: FnMut(&UnitBaseName) -> Option<ResolvedUnit>,
{
    units
        .iter()
        .filter_map(|unit| {
            let base_name = match unit.info() {
                UnitInfo::Standard { base_name, .. } => Some(base_name),
                UnitInfo::Db { base_name, .. } => base_name.as_ref(),
            }?;
            let resolved = lookup_unit(base_name)?;
            Some(resolved.dimension_map.pow(unit.exponent()))
        })
        .fold(DimensionMap::dimensionless(), |acc, dim| acc * dim)
}

/// Information about a unit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UnitInfo {
    /// A standard unit
    Standard {
        /// The prefix of the unit, if any
        prefix: Option<UnitPrefix>,
        /// The stripped name of the unit, if any
        base_name: UnitBaseName,
    },

    /// A decibel unit
    Db {
        /// The prefix of the unit, if any
        prefix: Option<UnitPrefix>,
        /// The stripped name of the unit, if any
        base_name: Option<UnitBaseName>,
    },
}

/// A unit used for displaying the unit to
/// the user.
///
/// This retains multiplication and division and
/// the original exponent, rather than converting
/// it to a list of units that are multiplied
/// together.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum DisplayCompositeUnit {
    /// Multiplied units
    Multiply(Box<Self>, Box<Self>),
    /// Divided units
    Divide(Box<Self>, Box<Self>),
    /// A single unit
    BaseUnit(DisplayUnit),
    /// `1` unit
    One,
}

impl DisplayCompositeUnit {
    /// Lowers this AST-style display unit into the runtime
    /// [`ResolvedDisplayUnit`] used by [`oneil_output::Unit`] and error
    /// messages. Pure data conversion — no symbol lookup required.
    #[must_use]
    pub fn to_resolved_display(&self) -> ResolvedDisplayUnit {
        match self {
            Self::BaseUnit(unit) => ResolvedDisplayUnit::Unit {
                name: unit.name.clone(),
                exponent: unit.exponent,
            },
            Self::One => ResolvedDisplayUnit::One,
            Self::Multiply(left, right) => left.to_resolved_display() * right.to_resolved_display(),
            Self::Divide(left, right) => left.to_resolved_display() / right.to_resolved_display(),
        }
    }
}

/// A unit used for displaying the unit to
/// the user.
///
/// This retains multiplication and division and
/// the original exponent, rather than converting
/// it to a list of units that are multiplied
/// together.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DisplayUnit {
    /// The name of the unit
    pub name: String,
    /// The exponent of the unit
    #[serde(with = "f64_serde")]
    pub exponent: f64,
}

impl DisplayUnit {
    /// Creates a new display unit with the specified name and exponent.
    #[must_use]
    pub const fn new(name: String, exponent: f64) -> Self {
        Self { name, exponent }
    }
}

/// Builders for unit test fixtures.
#[cfg(any(test, feature = "test-helpers"))]
pub mod test {
    use oneil_output::DimensionMap;
    use oneil_shared::{
        span::Span,
        symbols::{UnitBaseName, UnitName, UnitPrefix},
    };

    use super::{CompositeUnit, DisplayCompositeUnit, DisplayUnit, Unit, UnitInfo};

    /// Specification for a unit in tests.
    #[derive(Debug, Clone, Copy)]
    pub struct UnitSpec {
        /// Optional SI prefix.
        pub prefix: Option<&'static str>,
        /// Optional base-unit name.
        pub base_name: Option<&'static str>,
        /// Whether this is a decibel unit.
        pub is_db: bool,
        /// Unit exponent.
        pub exponent: f64,
    }

    impl UnitSpec {
        /// Creates a unit specification.
        #[must_use]
        pub const fn new(
            prefix: Option<&'static str>,
            base_name: Option<&'static str>,
            is_db: bool,
            exponent: f64,
        ) -> Self {
            Self {
                prefix,
                base_name,
                is_db,
                exponent,
            }
        }
    }

    /// Builds a unit's complete display name.
    fn build_full_name(base_name: Option<&str>, prefix: Option<&str>, is_db: bool) -> UnitName {
        UnitName::new(format!(
            "{}{}{}",
            if is_db { "dB" } else { "" },
            prefix.unwrap_or(""),
            base_name.unwrap_or("")
        ))
    }

    /// Builds resolved information for a unit specification.
    fn build_unit_info(base_name: Option<&str>, prefix: Option<&str>, is_db: bool) -> UnitInfo {
        if is_db {
            UnitInfo::Db {
                prefix: prefix.map(UnitPrefix::from),
                base_name: base_name.map(UnitBaseName::from),
            }
        } else {
            UnitInfo::Standard {
                prefix: prefix.map(UnitPrefix::from),
                base_name: UnitBaseName::from(base_name.expect("base name should be provided")),
            }
        }
    }

    /// Builds the display tree for a composite unit.
    fn display_composite_unit(
        unit_list: impl IntoIterator<Item = UnitSpec>,
    ) -> DisplayCompositeUnit {
        unit_list
            .into_iter()
            .map(|spec| {
                DisplayCompositeUnit::BaseUnit(DisplayUnit::new(
                    build_full_name(spec.base_name, spec.prefix, spec.is_db).into_string(),
                    spec.exponent,
                ))
            })
            .reduce(|left, right| DisplayCompositeUnit::Multiply(Box::new(left), Box::new(right)))
            .unwrap_or(DisplayCompositeUnit::One)
    }

    /// Builds an IR composite unit from unit specifications.
    #[must_use]
    pub fn ir_composite_unit(unit_list: impl IntoIterator<Item = UnitSpec>) -> CompositeUnit {
        let unit_specs: Vec<_> = unit_list.into_iter().collect();
        let display_unit = display_composite_unit(unit_specs.iter().copied());
        let units = unit_specs
            .into_iter()
            .map(|spec| {
                Unit::new(
                    Span::synthetic(),
                    build_full_name(spec.base_name, spec.prefix, spec.is_db),
                    Span::synthetic(),
                    spec.exponent,
                    None,
                    build_unit_info(spec.base_name, spec.prefix, spec.is_db),
                )
            })
            .collect();

        CompositeUnit::new(
            units,
            display_unit,
            Span::synthetic(),
            DimensionMap::dimensionless(),
        )
    }

    /// Builds optional resolved units, returning `None` when no specs are given.
    #[must_use]
    pub fn build_resolved_units(
        units: impl IntoIterator<Item = UnitSpec>,
    ) -> Option<CompositeUnit> {
        let units: Vec<_> = units.into_iter().collect();
        (!units.is_empty()).then(|| ir_composite_unit(units))
    }
}
