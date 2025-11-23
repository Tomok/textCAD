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

# With direnv (optional): environment auto-activates when entering directory
```

**Important**: The project is configured to use system Z3 (not compiled from source). The Nix environment provides both Rust toolchain and Z3 with proper environment variables (`Z3_SYS_Z3_HEADER`, `LIBCLANG_PATH`).

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

Example property: "For any positive length L, the solver finds a configuration where the line has exactly length L."

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
- Z3 crate version 0.12 with `default-features = false` to use system Z3
- Property-based testing with `proptest` crate
- Generational arena pattern for entity management
- CI/CD through GitHub Actions with Nix caching
- to mark checkboxes in markdown files as done use [x]
- before adding files to git, make sure to run formatting tools