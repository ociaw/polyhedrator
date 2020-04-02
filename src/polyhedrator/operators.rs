/// A Conway operator to apply to a polyhedron.
/// See [https://en.wikipedia.org/wiki/Conway_polyhedron_notation](Conway polyhedron notation) for
/// more information.
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub enum Operator {
    Ambo,
    Dual,
    Kis(Kis),
}

impl From<Operator> for String {
    fn from(operator: Operator) -> Self {
        match operator {
            Operator::Ambo => "a".into(),
            Operator::Dual => "d".into(),
            Operator::Kis(kis) => {
                if kis.side_count == 0 {
                    "k".into()
                }
                else {
                    format!("k{}", kis.side_count)
                }
            },
        }
    }
}

/// The `kis` operator (short for triakis, also known as [Kleetope](https://en.wikipedia.org/wiki/Kleetope))
/// replaces each n-sided face with a matching n-sided pyramid. For example, a hexagon becomes a
/// hexagonal pyramid.
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Kis {
    side_count: u32,
    apex_scale: f64,
}

impl Kis {
    /// Creates a `kis` operator that will only act on faces with the given number of sides.
    /// If `side_count` is zero, every side will be operated upon.
    pub fn restrict_to_sides(side_count: u32) -> Self {
        Kis {
            side_count,
            ..Default::default()
        }
    }

    /// Creates a `kis` operator with the given apex scale.
    /// # Restrictions
    /// Panics if `apex_scale` is NaN or infinite.
    pub fn scale_apex(apex_scale: f64) -> Self {
        assert!(!apex_scale.is_nan(), "Apex scale must not be NaN.");
        assert!(apex_scale.is_finite(), "Apex scale must be finite.");
        Kis {
            apex_scale,
            ..Default::default()
        }
    }

    /// Creates a `kis` operator with the given apex scale and will only act on faces with the given
    /// number of sides.
    /// # Restrictions
    /// Panics if `apex_scale` is NaN or infinite.
    pub fn restrict_to_sides_and_scale_apex(side_count: u32, apex_scale: f64) -> Self {
        assert!(!apex_scale.is_nan(), "Apex scale must not be NaN.");
        assert!(apex_scale.is_finite(), "Apex scale must be finite.");
        Kis {
            side_count,
            apex_scale,
        }
    }

    pub fn side_count(&self) -> u32 {
        self.side_count
    }

    /// This determines the height of the new apex for each affected face, by multiplying the scale
    /// by the average distance to the center for each vertex.
    /// This will always be finite and never a NaN.
    pub fn apex_scale(&self) -> f64 {
        self.apex_scale
    }
}

impl Default for Kis {
    fn default() -> Self {
        Kis {
            side_count: 0,
            apex_scale: 0.1,
        }
    }
}

use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "polyhedrator/notation.pest"]
struct NotationParser;

impl Operator {
    pub fn try_parse(value: &str) -> Result<Vec<Operator>, ()> {
        use pest::Parser;

        match NotationParser::parse(Rule::expression, value) {
            Ok(pairs) => {
                let mut operators = Vec::new();
                for expression_pair in pairs {
                    for operator_pair in expression_pair.into_inner() {
                        let operator = match operator_pair.as_rule() {
                            Rule::ambo => Operator::Ambo,
                            Rule::dual => Operator::Dual,
                            Rule::kis => {
                                let parameter_pair = operator_pair.into_inner().next();
                                if let Some(parameter_pair) = parameter_pair {
                                    assert_eq!(parameter_pair.as_rule(), Rule::parameter);
                                    let sides = parameter_pair.as_str().parse::<u32>().unwrap();
                                    Operator::Kis(Kis::restrict_to_sides_and_scale_apex(sides, 0.0))
                                }
                                else {
                                    Operator::Kis(Kis::scale_apex(0.0))
                                }
                            },
                            _ => unreachable!(),
                        };
                        operators.push(operator);
                    }
                }
                Ok(operators)
            },
            Err(_) => Err(())
        }
    }
}
