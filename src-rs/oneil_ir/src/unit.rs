//! Unit system for dimensional analysis in Oneil.

use oneil_output::{DimensionMap, DisplayUnit as ResolvedDisplayUnit, Unit as ResolvedUnit};
use oneil_shared::{
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
#[derive(Debug, Clone, PartialEq)]
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
    pub exponent: f64,
}

impl DisplayUnit {
    /// Creates a new display unit with the specified name and exponent.
    #[must_use]
    pub const fn new(name: String, exponent: f64) -> Self {
        Self { name, exponent }
    }
}
