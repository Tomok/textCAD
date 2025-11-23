# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TextCAD is a constraint-based 2D CAD system built in Rust using Z3 as the constraint solver. The system provides a declarative API for geometric constraint specification while leveraging industrial-strength satisfiability technology for solving.

## Development Environment

This project uses Nix flakes for reproducible development environments:

```bash
# Enter development environment (includes Rust + Z3)
nix develop

# Build the project
cargo build

# Run tests (includes property-based tests with proptest)
cargo test

# Lint and format
cargo clippy
cargo fmt

# Generate code coverage reports
cargo llvm-cov --all-features --workspace --html  # HTML report in target/llvm-cov/html/
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info  # LCOV format
cargo llvm-cov --all-features --workspace --open  # Generate and open HTML report

# With direnv (optional): environment auto-activates when entering directory
```

**Important**: The project supports two Z3 configuration modes:
- **System Z3 (default)**: Uses Z3 installed on your system. Faster to compile, requires Z3 to be installed.
- **Vendored Z3**: Builds and statically links Z3 from source. Slower to compile but doesn't require system Z3.

The Nix environment provides both Rust toolchain and system Z3 with proper environment variables (`Z3_SYS_Z3_HEADER`, `LIBCLANG_PATH`).

```bash
# Build with system Z3 (default, requires Z3 installed)
cargo build

# Build with vendored Z3 (builds Z3 from source)
cargo build --features vendored-z3

# Run tests with vendored Z3
cargo test --features vendored-z3
```

## Git Hooks

The repository includes git hooks to maintain code quality:

```bash
# Install git hooks (one-time setup)
./hooks/install-hooks.sh
```

The pre-commit hook performs the following checks and **will reject commits** if they fail:
- **Code formatting**: Ensures code is formatted with `cargo fmt` (always runs)
- **Tests**: Runs `cargo test` to verify all tests pass (requires Nix environment)

**Important**: The hook will block commits if:
- Code is not properly formatted
- Tests fail
- Tests cannot run (Z3 environment not available)

To temporarily skip tests (e.g., when working outside Nix environment):
```bash
SKIP_TESTS=1 git commit -m "your message"
```

To bypass hooks entirely (not recommended):
```bash
git commit --no-verify -m "your message"
```

## Code Coverage

The project uses `cargo-llvm-cov` for code coverage reporting:

```bash
# Generate HTML coverage report (viewable in browser)
cargo llvm-cov --all-features --workspace --html
# Report saved to: target/llvm-cov/html/index.html

# Generate and automatically open HTML report
cargo llvm-cov --all-features --workspace --open

# Generate LCOV format (for CI/coverage services)
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate JSON format
cargo llvm-cov --all-features --workspace --json --output-path coverage.json
```

Coverage reports are automatically generated and uploaded to Codecov on every push to main and pull request via the CI pipeline.

**Coverage Goals**:
- Aim for >80% overall code coverage
- All public APIs should have test coverage
- Property-based tests help ensure edge cases are covered

## Architecture Overview

The system follows a constraint-based parametric design approach with clear separation between constraint specification and solving:

- **Sketch Manager**: Maintains collections of geometric entities and constraints
- **Entity Arena**: Stores geometric entities (Point2D, Line, Circle) using generational arenas with strongly-typed IDs
- **Constraint System**: Trait-based constraints that translate geometric relationships into Z3 assertions
- **Z3 Integration**: Uses Z3's theory of real arithmetic for constraint solving
- **Solution Extraction**: Converts Z3 rational results to floating-point coordinates
- **Export System**: Transforms solved configurations to SVG (with future STL support planned)

### Key Design Patterns

- **Entity-as-Constraint-Factory**: Geometric entities provide methods that return constraint objects (e.g., `line.contains_point(point_id)` returns `PointOnLineConstraint`)
- **Arena-Based References**: Uses typed IDs (`PointId`, `LineId`) instead of Rust references to avoid lifetime complexity
- **Declarative API**: Users specify relationships (what should hold) rather than computations (how to achieve them)
- **Immutable Constraints**: Once fully described, geometry and constraints are immutable

### Unit System

The system uses newtype wrappers for type-safe physical quantities:
- `Length` (stored in meters, provides `.meters()`, `.millimeters()`, etc.)
- `Angle` (stored in radians, provides `.radians()`, `.degrees()`, etc.)

### Constraint Solving

- Uses Z3's rational arithmetic during solving for exactness
- Angular constraints use unit circle method (dot/cross products) to avoid transcendental functions
- Parametric constraints (e.g., point-on-line) introduce internal parameter variables automatically
- Batch constraint application: all constraints applied simultaneously in single solve phase

## Implementation Phases

The project follows a phased implementation approach (see docs/IMPLEMENTATION_PLAN.md for details and docs/ARCHITECTURE.md for architectural documentation):

**Phase 1** ✅: Infrastructure (Nix + CI) - COMPLETED
**Phase 2**: Rust Foundation (units, error types, traits)
**Phase 3**: Z3 Integration & Context
**Phase 4**: Point2D entity
**Phase 5**: Basic constraints (coincident, fixed position)
**Phase 6**: Solution extraction
**Phase 7**: Line entity
**Phase 8**: Line constraints (length, parallel, perpendicular)
**Phase 9**: Circle entity
**Phase 10**: Parametric constraints
**Phase 11**: Angle constraints
**Phase 12**: SVG export

Each phase includes unit tests, integration tests, and property-based tests using `proptest`.

## Testing Strategy

The project employs multiple testing approaches:
- **Unit Tests**: Individual component testing
- **Integration Tests**: Complete workflows (create sketch, add constraints, solve, extract)
- **Property-Based Tests**: Using `proptest` to verify properties hold for random inputs
- **Code Coverage**: Measured with `cargo-llvm-cov`, tracked in CI, aiming for >80% coverage

Example property: "For any positive length L, the solver finds a configuration where the line has exactly length L."

Coverage reports help identify untested code paths and ensure comprehensive test suites.

## Key Files and Modules

- `src/lib.rs`: Main library entry point
- `src/units.rs`: Unit system with Length/Angle types (Phase 2)
- `src/error.rs`: Error types and Result aliases (Phase 2)
- `src/sketch.rs`: Main sketch management and Z3 context (Phase 3+)
- `src/entities/`: Geometric entity types (Point2D, Line, Circle)
- `src/constraints/`: Constraint implementations
- `src/solution.rs`: Solution extraction and coordinate conversion
- `src/export/`: Export system (SVG, future STL)

## Development Notes

- Rust edition 2024 is used
- Z3 crate version 0.12 with two configuration modes:
  - **Default**: Uses system Z3 (`default-features = false`)
  - **vendored-z3 feature**: Builds Z3 from source (`static-link-z3` feature)
- Property-based testing with `proptest` crate
- Generational arena pattern for entity management
- CI/CD through GitHub Actions with Nix caching
- to mark checkboxes in markdown files as done use [x]
- before adding files to git, make sure to run formatting tools