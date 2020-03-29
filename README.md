# Polyhedrator
Polyhedrator is a polyhedron generator and viewer. It uses [Conway Polyhedron Notation](https://en.wikipedia.org/wiki/Conway_polyhedron_notation)
to represent transformations on polyhedrons.

Polyhedrons can be seeded from the 5 Platonic solids:
* Tetrahedron
* Cube
* Octahedron
* Dodecahedron
* Icosahedron

Then each operator is applied right to left. Currently, the following operators are supported:
* Ambo
* Dual
* Kis (n)

Kis may take a parameter `n`, which restricts it to only operating on faces with `n` number of sides.
Several other operators can be constructed with this set, such as truncation: `dkd`.

## Running
Install [Rust](https://www.rust-lang.org/), clone this repository, and execute `cargo run`.

## Platform Support
Tested only on Windows 10, however since it uses WebGPU for rendering, it *should* `Just Work` on Linux and macOS.
