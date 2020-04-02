mod platonic;

pub use platonic::Platonic;

use super::{Face, Polyhedron};

#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub enum Seed {
    Platonic(Platonic),
}

impl Seed {
    pub fn polyhedron(self, edge_length: f64) -> Polyhedron {
        match self {
            Seed::Platonic(platonic) => Platonic::polyhedron(platonic, edge_length)
        }
    }
}

impl From<Seed> for &str {
    fn from(seed: Seed) -> &'static str {
        match seed {
            Seed::Platonic(platonic) => platonic.into()
        }
    }
}

impl std::fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Seed::Platonic(platonic) => platonic.fmt(f)
        }
    }
}
