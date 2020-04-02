use super::{Face, Polyhedron};

#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub enum Platonic {
    Tetrahedron,
    Cube,
    Octahedron,
    Dodecahedron,
    Icosahedron,
}

fn golden_ratio() -> f64 {
    (1.0 + 5.0f64.sqrt()) / 2.0
}

impl Platonic {
    pub fn all() -> [Platonic; 5] {
        [
            Platonic::Tetrahedron,
            Platonic::Cube,
            Platonic::Octahedron,
            Platonic::Dodecahedron,
            Platonic::Icosahedron,
        ]
    }

    pub fn polyhedron(self, edge_length: f64) -> Polyhedron {
        match self {
            Platonic::Tetrahedron => Self::tetrahedron(edge_length),
            Platonic::Cube => Self::cube(edge_length),
            Platonic::Octahedron => Self::octahedron(edge_length),
            Platonic::Dodecahedron => Self::dodecahedron(edge_length),
            Platonic::Icosahedron => Self::icosahedron(edge_length),
        }
    }

    pub fn tetrahedron(edge_length: f64) -> Polyhedron {
        let scalar = edge_length / 2.0;
        let sqrt = std::f64::consts::FRAC_1_SQRT_2 * scalar;

        Polyhedron {
            vertices: vec![
                [scalar, 0.0, -sqrt].into(),
                [-scalar, 0.0, -sqrt].into(),
                [0.0, scalar, sqrt].into(),
                [0.0, -scalar, sqrt].into(),
            ],
            faces: vec![
                Face {
                    indices: vec![0, 1, 2],
                },
                Face {
                    indices: vec![0, 2, 3],
                },
                Face {
                    indices: vec![0, 3, 1],
                },
                Face {
                    indices: vec![1, 3, 2],
                },
            ],
        }
    }

    pub fn cube(edge_length: f64) -> Polyhedron {
        let scalar = edge_length / 2.0;

        Polyhedron {
            vertices: vec![
                [scalar, scalar, scalar].into(),
                [scalar, scalar, -scalar].into(),
                [scalar, -scalar, scalar].into(),
                [scalar, -scalar, -scalar].into(),
                [-scalar, scalar, scalar].into(),
                [-scalar, scalar, -scalar].into(),
                [-scalar, -scalar, scalar].into(),
                [-scalar, -scalar, -scalar].into(),
            ],
            faces: vec![
                Face {
                    indices: vec![0, 2, 3, 1],
                },
                Face {
                    indices: vec![4, 5, 7, 6],
                },
                Face {
                    indices: vec![0, 1, 5, 4],
                },
                Face {
                    indices: vec![2, 6, 7, 3],
                },
                Face {
                    indices: vec![0, 4, 6, 2],
                },
                Face {
                    indices: vec![1, 3, 7, 5],
                },
            ],
        }
    }

    pub fn octahedron(edge_length: f64) -> Polyhedron {
        let scalar = edge_length / std::f64::consts::SQRT_2;

        Polyhedron {
            vertices: vec![
                [scalar, 0.0, 0.0].into(),
                [-scalar, 0.0, 0.0].into(),
                [0.0, scalar, 0.0].into(),
                [0.0, -scalar, 0.0].into(),
                [0.0, 0.0, scalar].into(),
                [0.0, 0.0, -scalar].into(),
            ],
            faces: vec![
                Face {
                    indices: vec![0, 2, 4],
                },
                Face {
                    indices: vec![0, 4, 3],
                },
                Face {
                    indices: vec![0, 3, 5],
                },
                Face {
                    indices: vec![0, 5, 2],
                },
                Face {
                    indices: vec![1, 4, 2],
                },
                Face {
                    indices: vec![1, 2, 5],
                },
                Face {
                    indices: vec![1, 5, 3],
                },
                Face {
                    indices: vec![1, 3, 4],
                },
            ],
        }
    }

    pub fn dodecahedron(edge_length: f64) -> Polyhedron {
        // Thank goodness for CAS
        let scalar = (edge_length * (5.0f64.sqrt() + 1.0)) / 4.0;
        let phi = (edge_length * (5.0f64.sqrt() + 3.0)) / 4.0;
        let inverse = edge_length / 2.0;

        Polyhedron {
            vertices: vec![
                [scalar, scalar, scalar].into(),
                [scalar, scalar, -scalar].into(),
                [scalar, -scalar, scalar].into(),
                [scalar, -scalar, -scalar].into(),
                [-scalar, scalar, scalar].into(),
                [-scalar, scalar, -scalar].into(),
                [-scalar, -scalar, scalar].into(),
                [-scalar, -scalar, -scalar].into(),
                [0.0, phi, inverse].into(),
                [0.0, phi, -inverse].into(),
                [0.0, -phi, inverse].into(),
                [0.0, -phi, -inverse].into(),
                [phi, inverse, 0.0].into(),
                [phi, -inverse, 0.0].into(),
                [-phi, inverse, 0.0].into(),
                [-phi, -inverse, 0.0].into(),
                [inverse, 0.0, phi].into(),
                [-inverse, 0.0, phi].into(),
                [inverse, 0.0, -phi].into(),
                [-inverse, 0.0, -phi].into(),
            ],
            faces: vec![
                Face {
                    indices: vec![0, 8, 4, 17, 16],
                },
                Face {
                    indices: vec![0, 12, 1, 9, 8],
                },
                Face {
                    indices: vec![0, 16, 2, 13, 12],
                },
                Face {
                    indices: vec![1, 12, 13, 3, 18],
                },
                Face {
                    indices: vec![1, 18, 19, 5, 9],
                },
                Face {
                    indices: vec![2, 10, 11, 3, 13],
                },
                Face {
                    indices: vec![3, 11, 7, 19, 18],
                },
                Face {
                    indices: vec![4, 8, 9, 5, 14],
                },
                Face {
                    indices: vec![4, 14, 15, 6, 17],
                },
                Face {
                    indices: vec![5, 19, 7, 15, 14],
                },
                Face {
                    indices: vec![6, 10, 2, 16, 17],
                },
                Face {
                    indices: vec![6, 15, 7, 11, 10],
                },
            ],
        }
    }

    pub fn icosahedron(edge_length: f64) -> Polyhedron {
        let scalar = edge_length / 2.0;
        let phi = golden_ratio() * scalar;

        Polyhedron {
            vertices: vec![
                [-scalar, phi, 0.0].into(),
                [scalar, phi, 0.0].into(),
                [-scalar, -phi, 0.0].into(),
                [scalar, -phi, 0.0].into(),
                [0.0, -scalar, phi].into(),
                [0.0, scalar, phi].into(),
                [0.0, -scalar, -phi].into(),
                [0.0, scalar, -phi].into(),
                [phi, 0.0, -scalar].into(),
                [phi, 0.0, scalar].into(),
                [-phi, 0.0, -scalar].into(),
                [-phi, 0.0, scalar].into(),
            ],
            faces: vec![
                Face {
                    indices: vec![0, 1, 7],
                },
                Face {
                    indices: vec![0, 5, 1],
                },
                Face {
                    indices: vec![0, 7, 10],
                },
                Face {
                    indices: vec![0, 10, 11],
                },
                Face {
                    indices: vec![0, 11, 5],
                },
                Face {
                    indices: vec![1, 5, 9],
                },
                Face {
                    indices: vec![1, 8, 7],
                },
                Face {
                    indices: vec![1, 9, 8],
                },
                Face {
                    indices: vec![2, 3, 4],
                },
                Face {
                    indices: vec![2, 4, 11],
                },
                Face {
                    indices: vec![2, 6, 3],
                },
                Face {
                    indices: vec![2, 10, 6],
                },
                Face {
                    indices: vec![2, 11, 10],
                },
                Face {
                    indices: vec![3, 6, 8],
                },
                Face {
                    indices: vec![3, 8, 9],
                },
                Face {
                    indices: vec![3, 9, 4],
                },
                Face {
                    indices: vec![4, 5, 11],
                },
                Face {
                    indices: vec![4, 9, 5],
                },
                Face {
                    indices: vec![6, 7, 8],
                },
                Face {
                    indices: vec![6, 10, 7],
                },
            ],
        }
    }
}

impl From<Platonic> for &str {
    fn from(platonic: Platonic) -> &'static str {
        match platonic {
            Platonic::Tetrahedron => "T",
            Platonic::Cube => "C",
            Platonic::Octahedron => "O",
            Platonic::Dodecahedron => "D",
            Platonic::Icosahedron => "I",
        }
    }
}

impl std::fmt::Display for Platonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Platonic::Tetrahedron => "Tetrahedron",
            Platonic::Cube => "Cube",
            Platonic::Octahedron => "Octahedron",
            Platonic::Dodecahedron => "Dodecahedron",
            Platonic::Icosahedron => "Icosahedron",
        })
    }
}
