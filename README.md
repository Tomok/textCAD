# TextCAD

[![CI](https://github.com/Tomok/textCAD/actions/workflows/ci.yml/badge.svg)](https://github.com/Tomok/textCAD/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Tomok/textCAD/branch/main/graph/badge.svg)](https://codecov.io/gh/Tomok/textCAD)
[![Rust Edition](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A constraint-based 2D CAD system built in Rust using Z3 as the constraint solver. TextCAD provides a declarative API for geometric constraint specification while leveraging industrial-strength satisfiability technology for solving.

## Features

- **Constraint-Based Modeling**: Specify geometric relationships declaratively (what should hold) rather than imperatively (how to achieve them)
- **Z3 Integration**: Uses Z3's theory of real arithmetic for robust constraint solving
- **Type-Safe Units**: Newtype wrappers for physical quantities (Length, Angle) with automatic conversions
- **Arena-Based Entity Management**: Strongly-typed IDs prevent use-after-free bugs
- **Geometric Primitives**: Points, Lines, and Circles with comprehensive constraint support
- **Property-Based Testing**: Uses `proptest` to verify geometric properties across random inputs
- **Code Coverage**: Comprehensive test suite with >80% code coverage tracked via Codecov

## Architecture

TextCAD follows a constraint-based parametric design approach:

- **Sketch Manager**: Maintains collections of geometric entities and constraints
- **Entity Arena**: Stores geometric entities using generational arenas with strongly-typed IDs
- **Constraint System**: Trait-based constraints that translate geometric relationships into Z3 assertions
- **Solution Extraction**: Converts Z3 rational results to floating-point coordinates
- **Export System**: Transforms solved configurations to SVG (STL support planned)

### Key Design Patterns

- **Entity-as-Constraint-Factory**: Geometric entities provide methods that return constraint objects
  ```rust
  line.length_equals(Length::meters(5.0))
  line.parallel_to(&other_line)
  ```
- **Arena-Based References**: Uses typed IDs (`PointId`, `LineId`) instead of Rust references
- **Declarative API**: Users specify relationships rather than computations
- **Immutable Constraints**: Geometry and constraints are immutable once fully described

## Installation

### Prerequisites

This project uses Nix flakes for reproducible development environments. You'll need:

- [Nix](https://nixos.org/download.html) with flakes enabled
- (Optional) [direnv](https://direnv.net/) for automatic environment activation

### Enable Nix Flakes

If you haven't already enabled flakes, add this to `~/.config/nix/nix.conf`:

```conf
experimental-features = nix-command flakes
```

### Using Nix

#### Option 1: Manual Environment (Recommended for trying it out)

Enter the development environment manually:

```bash
# Clone the repository
git clone https://github.com/Tomok/textCAD.git
cd textCAD

# Enter development environment (downloads Rust + Z3)
nix develop

# Now you have access to all tools
rustc --version
z3 --version

# Build the project
cargo build

# Run tests
cargo test

# Exit the environment when done
exit
```

#### Option 2: Automatic Environment with direnv (Recommended for development)

For automatic environment activation when entering the directory:

```bash
# One-time setup
cd textCAD
echo "use flake" > .envrc
direnv allow

# Now whenever you cd into the directory, the environment activates automatically
cd textCAD  # Environment loads automatically
cargo build  # All tools ready to use
```

#### Option 3: Build without entering shell

You can build the project directly without entering the development shell:

```bash
# Build the project
nix build

# Run the built binary
./result/bin/textcad
```

### What Nix Provides

The Nix environment includes:

- **Rust toolchain** (stable, with rust-analyzer and rust-src)
- **Z3 SMT solver** (system Z3, not compiled from source)
- **Build tools** (pkg-config, cargo, rustc)
- **Proper environment variables** (`Z3_SYS_Z3_HEADER`, `LIBCLANG_PATH`)

This ensures everyone has the exact same development environment, regardless of their operating system.

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for a specific module
cargo test constraints::
```

### Code Coverage

Generate and view code coverage reports:

```bash
# Generate HTML coverage report
cargo llvm-cov --all-features --workspace --html

# View report in browser
cargo llvm-cov --all-features --workspace --open

# Generate LCOV format (for CI/coverage services)
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

Coverage reports are automatically generated and uploaded to [Codecov](https://codecov.io/gh/Tomok/textCAD) on every push to main and pull request.

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Run linter
cargo clippy

# Run linter with warnings as errors
cargo clippy -- -D warnings
```

### Git Hooks

The repository includes git hooks to maintain code quality:

```bash
# Install git hooks (one-time setup)
./hooks/install-hooks.sh
```

The pre-commit hook automatically:
- Checks code formatting with `cargo fmt`
- Runs tests with `cargo test`

To temporarily skip tests (when working outside Nix environment):
```bash
SKIP_TESTS=1 git commit -m "your message"
```

## Usage Example

```rust
use textcad::*;

fn main() {
    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create points
    let p1 = sketch.add_point(Some("P1"));
    let p2 = sketch.add_point(Some("P2"));
    let p3 = sketch.add_point(Some("P3"));

    // Fix first point at origin
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Create a line
    let line = sketch.add_line(p1, p2, Some("Base"));

    // Add constraints using entity-as-constraint-factory pattern
    sketch.add_constraint(line.length_equals(Length::meters(5.0)));

    // Solve and extract solution
    let solution = sketch.solve_and_extract().unwrap();

    // Get coordinates
    let (x, y) = solution.get_point_coordinates(p2).unwrap();
    println!("P2 is at ({:.2}, {:.2}) meters", x, y);
}
```

For more examples, see the `examples/` directory.

## Project Status

The project is under active development following a phased implementation plan:

- ✅ **Phase 1**: Infrastructure (Nix + CI)
- ✅ **Phase 2**: Rust Foundation (units, error types, traits)
- ✅ **Phase 3**: Z3 Integration & Context
- ✅ **Phase 4**: Point2D entity
- ✅ **Phase 5**: Basic constraints (coincident, fixed position)
- ✅ **Phase 6**: Solution extraction
- ✅ **Phase 7**: Line entity
- ✅ **Phase 8**: Line constraints (length, parallel, perpendicular)
- ✅ **Phase 9**: Circle entity
- ✅ **Phase 10**: Parametric constraints (point-on-line)
- ✅ **Phase 11**: Angle constraints
- ⏳ **Phase 12**: SVG export
- 🔮 **Future**: View system, Bézier curves, complex constraints

See [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) for detailed phase information.

## Documentation

- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md) - Detailed phase-by-phase implementation guide
- [Architecture Overview](docs/ARCHITECTURE.md) - System architecture and design patterns
- [Development Guide](CLAUDE.md) - Guide for developers and AI assistants

## Testing Strategy

TextCAD employs multiple testing approaches:

- **Unit Tests**: Individual component testing
- **Integration Tests**: Complete workflows (create sketch, add constraints, solve, extract)
- **Property-Based Tests**: Using `proptest` to verify properties hold for random inputs
- **Code Coverage**: Measured with `cargo-llvm-cov`, tracked in CI, aiming for >80% coverage

Example property: "For any positive length L, the solver finds a configuration where the line has exactly length L."

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Git hooks are installed: `./hooks/install-hooks.sh`

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

- Built with [Z3 Theorem Prover](https://github.com/Z3Prover/z3)
- Uses [Nix](https://nixos.org/) for reproducible builds
- Inspired by constraint-based CAD systems like SolveSpace and FreeCAD's Sketcher
