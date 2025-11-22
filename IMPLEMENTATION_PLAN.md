# TextCAD Implementation Plan

## Overview

This document outlines the phased implementation plan for TextCAD, a constraint-based 2D CAD system built in Rust using Z3 as the constraint solver. The implementation is structured in incremental phases, each independently testable through unit tests, integration tests, and property-based tests.

## Project Goals

- Develop 2D geometric primitives (Point, Line, Circle)
- Implement constraint-based parametric design
- Use Z3 theorem prover for constraint solving
- Support SI units with type safety
- Export to SVG format
- Maintain reproducible builds via Nix flakes
- Achieve fast CI/CD through effective caching

## Testing Strategy

### Unit Tests
Test individual components in isolation (e.g., unit conversions, entity creation).

### Integration Tests
Test complete workflows (e.g., create sketch, add constraints, solve, extract solution).

### Property-Based Tests
Test general properties that must hold for randomly generated inputs.

**Example: Line Length Constraint**

Classic unit test:
```rust
#[test]
fn test_line_length_constraint() {
    let mut sketch = Sketch::new();
    let p1 = sketch.add_point(0.0, 0.0);
    let p2 = sketch.add_point(3.0, 4.0);
    let line = sketch.add_line(p1, p2);
    
    sketch.add_constraint(line.length_equals(Length::meters(5.0)));
    
    let solution = sketch.solve().unwrap();
    assert_eq!(solution.line_length(line), 5.0);
}
```

Property-based test:
```rust
#[quickcheck]
fn prop_line_length_constraint_always_satisfied(
    target_length: f64  // randomly generated
) -> bool {
    let target_length = target_length.abs().max(0.001); // positive length
    
    let mut sketch = Sketch::new();
    let p1 = sketch.add_point(0.0, 0.0);
    let p2 = sketch.add_point(1.0, 1.0);
    let line = sketch.add_line(p1, p2);
    
    sketch.add_constraint(line.length_equals(Length::meters(target_length)));
    
    if let Ok(solution) = sketch.solve() {
        let actual = solution.line_length(line);
        (actual - target_length).abs() < 0.0001  // Property: length is correct
    } else {
        false
    }
}
```

**Property**: "For any positive length L, the solver finds a configuration where the line has exactly length L."

### Additional Properties for TextCAD

1. **Idempotence**: Solving twice gives the same result
2. **Constraint Order Independence**: Order of constraints shouldn't change the solution
3. **Symmetry**: If P1-P2 has distance D, then P2-P1 also has distance D
4. **Triangle Inequality**: |P1P2| + |P2P3| ≥ |P1P3|

---

## Implementation Phases

### Phase 1: Infrastructure (Nix Flakes + CI)

**Deliverables:**
- `flake.nix` with reproducible build environment
- `Cargo.toml` configured to use system Z3 (not build from source)
- GitHub Actions workflow with effective caching
- Development environment setup documentation

**Files to Create:**

#### flake.nix
```nix
{
  description = "TextCAD - Constraint-based 2D/3D CAD system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        buildInputs = with pkgs; [
          z3  # System Z3 - won't be compiled by cargo!
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          
          # Environment variables for z3-sys crate
          Z3_SYS_Z3_HEADER = "${pkgs.z3.dev}/include/z3.h";
          
          shellHook = ''
            echo "TextCAD development environment"
            echo "Rust: $(rustc --version)"
            echo "Z3: $(z3 --version)"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "textcad";
          version = "0.1.0";
          src = ./.;
          
          cargoLock.lockFile = ./Cargo.lock;
          
          inherit nativeBuildInputs buildInputs;
          
          Z3_SYS_Z3_HEADER = "${pkgs.z3.dev}/include/z3.h";
        };
      }
    );
}
```

#### Cargo.toml
```toml
[package]
name = "textcad"
version = "0.1.0"
edition = "2021"

[dependencies]
# Z3 with static-link-z3 DISABLED to use system Z3
z3 = { version = "0.12", default-features = false }

[dev-dependencies]
proptest = "1.4"

[profile.dev]
opt-level = 1  # Faster dev builds with Z3
```

#### .github/workflows/ci.yml
```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      # Install Nix
      - uses: DeterminateSystems/nix-installer-action@main
      
      # Magic Nix Cache for fast builds
      - uses: DeterminateSystems/magic-nix-cache-action@main
      
      # Rust/Cargo Cache
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-
      
      # Build
      - name: Build
        run: nix develop --command cargo build --verbose
      
      # Tests
      - name: Run tests
        run: nix develop --command cargo test --verbose
      
      # Clippy
      - name: Clippy
        run: nix develop --command cargo clippy -- -D warnings
      
      # Format check
      - name: Format check
        run: nix develop --command cargo fmt -- --check
```

#### .envrc (Optional for direnv users)
```bash
use flake
```

#### .gitignore additions
```
/target
/result
.direnv/
```

**Tests:**
- [x] `nix flake check` succeeds
- [x] `nix develop --command bash -c "rustc --version && z3 --version"` shows versions
- [x] `nix develop --command cargo build` succeeds and uses system Z3
- [x] Verify Z3 is NOT compiled: `cargo clean && cargo build 2>&1 | grep -c "Compiling z3-sys"` should be 0
- [x] GitHub Actions workflow runs successfully (mock project)

**Documentation:**
- README.md with Nix setup instructions
- Local development workflow
- CI/CD pipeline explanation

---

### Phase 2: Rust Foundation ✅ COMPLETED

**Deliverables:**
- ✅ Unit system with newtype wrappers (Length, Angle, Area)
- ✅ Error types and Result aliases using thiserror
- ✅ Base traits for constraints and entity IDs
- ✅ Proper dimensional analysis (Length × Length = Area)

**Implementation:**

#### src/units.rs
```rust
/// Length in SI base unit (meters)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length(f64);

impl Length {
    pub fn meters(m: f64) -> Self {
        Self(m)
    }
    
    pub fn millimeters(mm: f64) -> Self {
        Self(mm / 1000.0)
    }
    
    pub fn centimeters(cm: f64) -> Self {
        Self(cm / 100.0)
    }
    
    pub fn as_meters(&self) -> f64 {
        self.0
    }
    
    pub fn as_millimeters(&self) -> f64 {
        self.0 * 1000.0
    }
}

/// Angle in SI unit (radians)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle(f64);

impl Angle {
    pub fn radians(rad: f64) -> Self {
        Self(rad)
    }
    
    pub fn degrees(deg: f64) -> Self {
        Self(deg * std::f64::consts::PI / 180.0)
    }
    
    pub fn as_radians(&self) -> f64 {
        self.0
    }
    
    pub fn as_degrees(&self) -> f64 {
        self.0 * 180.0 / std::f64::consts::PI
    }
}
```

#### src/error.rs
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextCADError {
    #[error("Constraint system is unsatisfiable")]
    Unsatisfiable,
    
    #[error("Invalid entity reference: {0}")]
    InvalidEntity(String),
    
    #[error("Solver error: {0}")]
    SolverError(String),
}

pub type Result<T> = std::result::Result<T, TextCADError>;
```

**Tests:**
- [x] Unit conversions: `Length::millimeters(1000.0) == Length::meters(1.0)`
- [x] Unit conversions: `Angle::degrees(180.0) == Angle::radians(PI)`
- [x] Angle conversions: 90°, 45°, 30° to radians and back
- [x] Error type construction and display
- [x] Dimensional analysis: Length × Length = Area
- [x] Area ÷ Length = Length

**Property-Based Tests:**
```rust
#[proptest]
fn prop_length_conversion_roundtrip(meters: f64) {
    let length = Length::meters(meters);
    prop_assert!((length.as_meters() - meters).abs() < 1e-10);
}

#[proptest]
fn prop_angle_degree_roundtrip(degrees: f64) {
    let angle = Angle::degrees(degrees);
    let back = angle.as_degrees();
    prop_assert!((back - degrees).abs() < 1e-10);
}
```

---

### Phase 3: Z3 Integration & Context

**Deliverables:**
- Z3 Context wrapper
- Sketch structure (empty, only contains context)
- Solver interface trait (for future abstraction)

**Implementation:**

#### src/sketch.rs
```rust
use z3::*;

pub struct Sketch<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
}

impl<'ctx> Sketch<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        let solver = Solver::new(ctx);
        Self { ctx, solver }
    }
    
    pub fn context(&self) -> &'ctx Context {
        self.ctx
    }
}
```

#### src/solver.rs (trait for future abstraction)
```rust
pub trait ConstraintSolver {
    type Solution;
    
    fn add_assertion(&mut self, assertion: impl Into<Assertion>);
    fn solve(&mut self) -> Result<Self::Solution>;
}
```

**Tests:**
- [ ] Create Z3 context and sketch
- [ ] Solve trivial equation: x + 2 = 5, extract x = 3
- [ ] Test unsatisfiable constraint: x > 5 AND x < 3

**Integration Test Example:**
```rust
#[test]
fn test_simple_z3_equation() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let sketch = Sketch::new(&ctx);
    
    let x = Real::new_const(sketch.context(), "x");
    let two = Real::from_real(sketch.context(), 2, 1);
    let five = Real::from_real(sketch.context(), 5, 1);
    
    sketch.solver.assert(&x.add(&two)._eq(&five));
    
    assert_eq!(sketch.solver.check(), SatResult::Sat);
    let model = sketch.solver.get_model().unwrap();
    let result = model.eval(&x, true).unwrap().as_real().unwrap();
    assert_eq!(result, (3, 1)); // 3/1 = 3
}
```

---

### Phase 4: Point2D - Simplest Entity

**Deliverables:**
- Point2D structure with x, y as Z3 Reals
- PointId newtype wrapper
- Arena for Points (using `generational-arena` crate)
- `Sketch::add_point()` method

**Implementation:**

#### Add dependency
```toml
[dependencies]
generational-arena = "0.2"
```

#### src/entities/point.rs
```rust
use z3::ast::Real;
use generational_arena::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointId(Index);

pub struct Point2D<'ctx> {
    pub id: PointId,
    pub x: Real<'ctx>,
    pub y: Real<'ctx>,
    pub name: Option<String>,
}

impl<'ctx> Point2D<'ctx> {
    pub fn new(
        id: PointId,
        ctx: &'ctx Context,
        name: Option<String>,
    ) -> Self {
        let x = Real::new_const(ctx, format!("{}_x", name.as_deref().unwrap_or("p")));
        let y = Real::new_const(ctx, format!("{}_y", name.as_deref().unwrap_or("p")));
        
        Self { id, x, y, name }
    }
}
```

#### Update src/sketch.rs
```rust
use generational_arena::Arena;

pub struct Sketch<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
    points: Arena<Point2D<'ctx>>,
}

impl<'ctx> Sketch<'ctx> {
    pub fn add_point(&mut self, name: Option<String>) -> PointId {
        let idx = self.points.insert_with(|idx| {
            let id = PointId(idx);
            Point2D::new(id, self.ctx, name)
        });
        PointId(idx)
    }
    
    pub fn get_point(&self, id: PointId) -> Option<&Point2D<'ctx>> {
        self.points.get(id.0)
    }
}
```

**Tests:**
- [ ] Create point with ID
- [ ] Retrieve point by ID
- [ ] Multiple points have distinct IDs
- [ ] Point variables have correct names in Z3

**Integration Test:**
```rust
#[test]
fn test_point_creation() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let p1 = sketch.add_point(Some("p1".into()));
    let p2 = sketch.add_point(Some("p2".into()));
    
    assert_ne!(p1, p2);
    assert!(sketch.get_point(p1).is_some());
    assert!(sketch.get_point(p2).is_some());
}
```

---

### Phase 5: Basic Constraint System

**Deliverables:**
- Constraint trait
- CoincidentPoints constraint (P1 = P2)
- PointAtPosition constraint (P fixed at x, y)
- `Sketch::add_constraint()`
- `Sketch::solve()` - first version

**Implementation:**

#### src/constraints/mod.rs
```rust
use crate::sketch::Sketch;
use crate::error::Result;

pub trait Constraint {
    fn apply<'ctx>(&self, sketch: &Sketch<'ctx>) -> Result<()>;
}

pub struct CoincidentPoints {
    pub p1: PointId,
    pub p2: PointId,
}

impl Constraint for CoincidentPoints {
    fn apply<'ctx>(&self, sketch: &Sketch<'ctx>) -> Result<()> {
        let pt1 = sketch.get_point(self.p1)
            .ok_or_else(|| TextCADError::InvalidEntity("p1".into()))?;
        let pt2 = sketch.get_point(self.p2)
            .ok_or_else(|| TextCADError::InvalidEntity("p2".into()))?;
        
        sketch.solver.assert(&pt1.x._eq(&pt2.x));
        sketch.solver.assert(&pt1.y._eq(&pt2.y));
        
        Ok(())
    }
}

pub struct PointAtPosition {
    pub point: PointId,
    pub x: Length,
    pub y: Length,
}

impl Constraint for PointAtPosition {
    fn apply<'ctx>(&self, sketch: &Sketch<'ctx>) -> Result<()> {
        let pt = sketch.get_point(self.point)
            .ok_or_else(|| TextCADError::InvalidEntity("point".into()))?;
        
        let x_val = Real::from_real(sketch.context(), 
            (self.x.as_meters() * 1000.0) as i64, 1000);
        let y_val = Real::from_real(sketch.context(),
            (self.y.as_meters() * 1000.0) as i64, 1000);
        
        sketch.solver.assert(&pt.x._eq(&x_val));
        sketch.solver.assert(&pt.y._eq(&y_val));
        
        Ok(())
    }
}
```

#### Update src/sketch.rs
```rust
pub struct Sketch<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
    points: Arena<Point2D<'ctx>>,
    constraints: Vec<Box<dyn Constraint>>,
}

impl<'ctx> Sketch<'ctx> {
    pub fn add_constraint(&mut self, constraint: impl Constraint + 'static) {
        self.constraints.push(Box::new(constraint));
    }
    
    pub fn solve(&mut self) -> Result<Solution> {
        // Apply all constraints
        for constraint in &self.constraints {
            constraint.apply(self)?;
        }
        
        // Solve
        match self.solver.check() {
            SatResult::Sat => {
                let model = self.solver.get_model()
                    .ok_or(TextCADError::SolverError("No model".into()))?;
                Ok(Solution::new(model))
            }
            SatResult::Unsat => Err(TextCADError::Unsatisfiable),
            SatResult::Unknown => Err(TextCADError::SolverError("Unknown".into())),
        }
    }
}
```

**Tests:**
- [ ] Create constraint objects
- [ ] Fix point at position and solve
- [ ] Coincident points constraint
- [ ] Unsatisfiable system (point at two different positions)

**Property-Based Test:**
```rust
#[proptest]
fn prop_fixed_point_solution(x: f64, y: f64) {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let p = sketch.add_point(None);
    sketch.add_constraint(PointAtPosition {
        point: p,
        x: Length::meters(x),
        y: Length::meters(y),
    });
    
    let solution = sketch.solve()?;
    let coords = solution.get_point_coords(p)?;
    
    prop_assert!((coords.0 - x).abs() < 1e-6);
    prop_assert!((coords.1 - y).abs() < 1e-6);
}
```

---

### Phase 6: Solution Extraction

**Deliverables:**
- Solution structure
- Extract Point coordinates from Z3 Model
- Rational → f64 conversion

**Implementation:**

#### src/solution.rs
```rust
use z3::Model;
use std::collections::HashMap;

pub struct Solution<'ctx> {
    model: Model<'ctx>,
    point_coords: HashMap<PointId, (f64, f64)>,
}

impl<'ctx> Solution<'ctx> {
    pub fn new(model: Model<'ctx>) -> Self {
        Self {
            model,
            point_coords: HashMap::new(),
        }
    }
    
    pub fn extract_point(&mut self, point: &Point2D<'ctx>) -> Result<(f64, f64)> {
        if let Some(&coords) = self.point_coords.get(&point.id) {
            return Ok(coords);
        }
        
        let x_val = self.model.eval(&point.x, true)
            .ok_or(TextCADError::SolverError("Cannot eval x".into()))?;
        let y_val = self.model.eval(&point.y, true)
            .ok_or(TextCADError::SolverError("Cannot eval y".into()))?;
        
        let x = rational_to_f64(&x_val)?;
        let y = rational_to_f64(&y_val)?;
        
        self.point_coords.insert(point.id, (x, y));
        Ok((x, y))
    }
    
    pub fn get_point_coords(&self, id: PointId) -> Result<(f64, f64)> {
        self.point_coords.get(&id)
            .copied()
            .ok_or(TextCADError::InvalidEntity("Point not extracted".into()))
    }
}

fn rational_to_f64<'ctx>(ast: &Real<'ctx>) -> Result<f64> {
    if let Some((num, denom)) = ast.as_real() {
        Ok(num as f64 / denom as f64)
    } else {
        Err(TextCADError::SolverError("Not a rational".into()))
    }
}
```

**Tests:**
- [ ] Extract coordinates from solved point
- [ ] Rational conversion: 3/2 → 1.5
- [ ] Multiple points extracted correctly

**Property-Based Test:**
```rust
#[proptest]
fn prop_extracted_values_satisfy_constraints(x: f64, y: f64) {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let p = sketch.add_point(None);
    sketch.add_constraint(PointAtPosition {
        point: p,
        x: Length::meters(x),
        y: Length::meters(y),
    });
    
    let mut solution = sketch.solve()?;
    let point = sketch.get_point(p).unwrap();
    let (ex, ey) = solution.extract_point(point)?;
    
    // Verify extracted values satisfy constraint
    prop_assert!((ex - x).abs() < 1e-6);
    prop_assert!((ey - y).abs() < 1e-6);
}
```

---

### Phase 7: Line - First Composite Entity

**Deliverables:**
- Line structure (start_id, end_id)
- LineId newtype
- Arena for Lines
- `Sketch::add_line(p1, p2)`

**Implementation:**

#### src/entities/line.rs
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineId(Index);

pub struct Line {
    pub id: LineId,
    pub start: PointId,
    pub end: PointId,
    pub name: Option<String>,
}

impl Line {
    pub fn new(
        id: LineId,
        start: PointId,
        end: PointId,
        name: Option<String>,
    ) -> Self {
        Self { id, start, end, name }
    }
}
```

#### Update src/sketch.rs
```rust
pub struct Sketch<'ctx> {
    // ... existing fields
    lines: Arena<Line>,
}

impl<'ctx> Sketch<'ctx> {
    pub fn add_line(
        &mut self,
        start: PointId,
        end: PointId,
        name: Option<String>,
    ) -> LineId {
        let idx = self.lines.insert_with(|idx| {
            let id = LineId(idx);
            Line::new(id, start, end, name)
        });
        LineId(idx)
    }
    
    pub fn get_line(&self, id: LineId) -> Option<&Line> {
        self.lines.get(id.0)
    }
}
```

**Tests:**
- [ ] Create line referencing two points
- [ ] Line stores correct endpoint IDs
- [ ] Multiple lines with distinct IDs

**Integration Test:**
```rust
#[test]
fn test_line_with_fixed_endpoints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let p1 = sketch.add_point(Some("p1".into()));
    let p2 = sketch.add_point(Some("p2".into()));
    
    sketch.add_constraint(PointAtPosition {
        point: p1,
        x: Length::meters(0.0),
        y: Length::meters(0.0),
    });
    sketch.add_constraint(PointAtPosition {
        point: p2,
        x: Length::meters(3.0),
        y: Length::meters(4.0),
    });
    
    let line = sketch.add_line(p1, p2, Some("line1".into()));
    
    let mut solution = sketch.solve().unwrap();
    
    let p1_coords = solution.extract_point(sketch.get_point(p1).unwrap()).unwrap();
    let p2_coords = solution.extract_point(sketch.get_point(p2).unwrap()).unwrap();
    
    let length = ((p2_coords.0 - p1_coords.0).powi(2) 
                + (p2_coords.1 - p1_coords.1).powi(2)).sqrt();
    
    assert!((length - 5.0).abs() < 1e-6); // 3-4-5 triangle
}
```

---

### Phase 8: Line Constraints

**Deliverables:**
- LineLengthConstraint
- Helper methods for Line
- Optional: ParallelLines, PerpendicularLines (can be deferred)

**Implementation:**

#### src/constraints/line.rs
```rust
pub struct LineLengthConstraint {
    pub line: LineId,
    pub length: Length,
}

impl Constraint for LineLengthConstraint {
    fn apply<'ctx>(&self, sketch: &Sketch<'ctx>) -> Result<()> {
        let line = sketch.get_line(self.line)
            .ok_or_else(|| TextCADError::InvalidEntity("line".into()))?;
        
        let p1 = sketch.get_point(line.start)
            .ok_or_else(|| TextCADError::InvalidEntity("start".into()))?;
        let p2 = sketch.get_point(line.end)
            .ok_or_else(|| TextCADError::InvalidEntity("end".into()))?;
        
        // Distance squared: (x2-x1)² + (y2-y1)² = L²
        let dx = p2.x.sub(&p1.x);
        let dy = p2.y.sub(&p1.y);
        
        let dist_sq = dx.mul(&dx).add(&dy.mul(&dy));
        
        let target_meters = self.length.as_meters();
        let target_sq = Real::from_real(sketch.context(),
            (target_meters * target_meters * 1_000_000.0) as i64,
            1_000_000);
        
        sketch.solver.assert(&dist_sq._eq(&target_sq));
        
        Ok(())
    }
}
```

#### Helper methods for Solution
```rust
impl<'ctx> Solution<'ctx> {
    pub fn line_length(&mut self, sketch: &Sketch<'ctx>, line_id: LineId) -> Result<f64> {
        let line = sketch.get_line(line_id)
            .ok_or(TextCADError::InvalidEntity("line".into()))?;
        
        let p1 = sketch.get_point(line.start).unwrap();
        let p2 = sketch.get_point(line.end).unwrap();
        
        let (x1, y1) = self.extract_point(p1)?;
        let (x2, y2) = self.extract_point(p2)?;
        
        let length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        Ok(length)
    }
}
```

**Tests:**
- [ ] Line with specified length
- [ ] Line connecting two fixed points validates Pythagoras

**Property-Based Tests:**
```rust
#[proptest]
fn prop_line_length_arbitrary(target_length: f64) {
    let target_length = target_length.abs().max(0.001).min(100.0);
    
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);
    
    sketch.add_constraint(PointAtPosition {
        point: p1,
        x: Length::meters(0.0),
        y: Length::meters(0.0),
    });
    
    let line = sketch.add_line(p1, p2, None);
    sketch.add_constraint(LineLengthConstraint {
        line,
        length: Length::meters(target_length),
    });
    
    let mut solution = sketch.solve()?;
    let actual_length = solution.line_length(&sketch, line)?;
    
    prop_assert!((actual_length - target_length).abs() < 1e-4);
}

#[proptest]
fn prop_pythagorean_theorem(a: f64, b: f64) {
    let a = a.abs().max(0.1).min(10.0);
    let b = b.abs().max(0.1).min(10.0);
    let c = (a * a + b * b).sqrt();
    
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);
    
    sketch.add_constraint(PointAtPosition {
        point: p1,
        x: Length::meters(0.0),
        y: Length::meters(0.0),
    });
    sketch.add_constraint(PointAtPosition {
        point: p2,
        x: Length::meters(a),
        y: Length::meters(b),
    });
    
    let line = sketch.add_line(p1, p2, None);
    let mut solution = sketch.solve()?;
    let length = solution.line_length(&sketch, line)?;
    
    prop_assert!((length - c).abs() < 1e-4);
}
```

---

### Phase 9: Circle

**Deliverables:**
- Circle structure (center_id, radius)
- CircleId newtype
- Arena for Circles
- CircleRadiusConstraint
- PointOnCircleConstraint (parametric)

**Implementation:**

#### src/entities/circle.rs
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CircleId(Index);

pub struct Circle<'ctx> {
    pub id: CircleId,
    pub center: PointId,
    pub radius: Real<'ctx>,
    pub name: Option<String>,
}

impl<'ctx> Circle<'ctx> {
    pub fn new(
        id: CircleId,
        center: PointId,
        ctx: &'ctx Context,
        name: Option<String>,
    ) -> Self {
        let radius = Real::new_const(ctx, 
            format!("{}_radius", name.as_deref().unwrap_or("c")));
        
        Self { id, center, radius, name }
    }
}
```

#### src/constraints/circle.rs
```rust
pub struct CircleRadiusConstraint {
    pub circle: CircleId,
    pub radius: Length,
}

impl Constraint for CircleRadiusConstraint {
    fn apply<'ctx>(&self, sketch: &Sketch<'ctx>) -> Result<()> {
        let circle = sketch.get_circle(self.circle)
            .ok_or_else(|| TextCADError::InvalidEntity("circle".into()))?;
        
        let r_val = Real::from_real(sketch.context(),
            (self.radius.as_meters() * 1000.0) as i64, 1000);
        
        sketch.solver.assert(&circle.radius._eq(&r_val));
        
        Ok(())
    }
}

pub struct PointOnCircleConstraint {
    pub circle: CircleId,
    pub point: PointId,
}

impl Constraint for PointOnCircleConstraint {
    fn apply<'ctx>(&self, sketch: &Sketch<'ctx>) -> Result<()> {
        let circle = sketch.get_circle(self.circle)
            .ok_or_else(|| TextCADError::InvalidEntity("circle".into()))?;
        let point = sketch.get_point(self.point)
            .ok_or_else(|| TextCADError::InvalidEntity("point".into()))?;
        let center = sketch.get_point(circle.center)
            .ok_or_else(|| TextCADError::InvalidEntity("center".into()))?;
        
        // Distance from point to center equals radius
        // (px - cx)² + (py - cy)² = r²
        let dx = point.x.sub(&center.x);
        let dy = point.y.sub(&center.y);
        let dist_sq = dx.mul(&dx).add(&dy.mul(&dy));
        let r_sq = circle.radius.mul(&circle.radius);
        
        sketch.solver.assert(&dist_sq._eq(&r_sq));
        
        Ok(())
    }
}
```

**Tests:**
- [ ] Circle with fixed center and radius
- [ ] Point on circle satisfies distance constraint

**Property-Based Test:**
```rust
#[proptest]
fn prop_point_on_circle_distance(
    cx: f64, cy: f64, radius: f64
) {
    let radius = radius.abs().max(0.1).min(10.0);
    
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let center = sketch.add_point(None);
    sketch.add_constraint(PointAtPosition {
        point: center,
        x: Length::meters(cx),
        y: Length::meters(cy),
    });
    
    let circle = sketch.add_circle(center, None);
    sketch.add_constraint(CircleRadiusConstraint {
        circle,
        radius: Length::meters(radius),
    });
    
    let p = sketch.add_point(None);
    sketch.add_constraint(PointOnCircleConstraint {
        circle,
        point: p,
    });
    
    let mut solution = sketch.solve()?;
    let (px, py) = solution.extract_point(sketch.get_point(p).unwrap())?;
    let (ccx, ccy) = solution.extract_point(sketch.get_point(center).unwrap())?;
    
    let dist = ((px - ccx).powi(2) + (py - ccy).powi(2)).sqrt();
    
    prop_assert!((dist - radius).abs() < 1e-4);
}
```

---

### Phase 10: Parametric Constraints

**Deliverables:**
- PointOnLineConstraint with parameter t ∈ [0,1]
- Refactoring: Internal parameter variable management

**Implementation:**

#### src/constraints/parametric.rs
```rust
pub struct PointOnLineConstraint {
    pub line: LineId,
    pub point: PointId,
}

impl Constraint for PointOnLineConstraint {
    fn apply<'ctx>(&self, sketch: &Sketch<'ctx>) -> Result<()> {
        let line = sketch.get_line(self.line)
            .ok_or_else(|| TextCADError::InvalidEntity("line".into()))?;
        let point = sketch.get_point(self.point)
            .ok_or_else(|| TextCADError::InvalidEntity("point".into()))?;
        let p1 = sketch.get_point(line.start)
            .ok_or_else(|| TextCADError::InvalidEntity("start".into()))?;
        let p2 = sketch.get_point(line.end)
            .ok_or_else(|| TextCADError::InvalidEntity("end".into()))?;
        
        // Introduce parameter t
        let t = Real::new_const(sketch.context(), 
            format!("t_{}_{}", line.id.0.into_raw_parts().0, point.id.0.into_raw_parts().0));
        
        // Point = p1 + t * (p2 - p1)
        // px = p1.x + t * (p2.x - p1.x)
        // py = p1.y + t * (p2.y - p1.y)
        
        let dx = p2.x.sub(&p1.x);
        let dy = p2.y.sub(&p1.y);
        
        let px = p1.x.add(&t.mul(&dx));
        let py = p1.y.add(&t.mul(&dy));
        
        sketch.solver.assert(&point.x._eq(&px));
        sketch.solver.assert(&point.y._eq(&py));
        
        // Constrain t ∈ [0, 1]
        let zero = Real::from_real(sketch.context(), 0, 1);
        let one = Real::from_real(sketch.context(), 1, 1);
        sketch.solver.assert(&t.ge(&zero));
        sketch.solver.assert(&t.le(&one));
        
        Ok(())
    }
}
```

**Tests:**
- [ ] Point on line with t=0 is at start
- [ ] Point on line with t=1 is at end
- [ ] Point on line with t=0.5 is at midpoint

**Property-Based Test:**
```rust
#[proptest]
fn prop_point_on_line_parameter_bounds(
    x1: f64, y1: f64, x2: f64, y2: f64
) {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);
    sketch.add_constraint(PointAtPosition {
        point: p1,
        x: Length::meters(x1),
        y: Length::meters(y1),
    });
    sketch.add_constraint(PointAtPosition {
        point: p2,
        x: Length::meters(x2),
        y: Length::meters(y2),
    });
    
    let line = sketch.add_line(p1, p2, None);
    let p = sketch.add_point(None);
    sketch.add_constraint(PointOnLineConstraint { line, point: p });
    
    let mut solution = sketch.solve()?;
    let (px, py) = solution.extract_point(sketch.get_point(p).unwrap())?;
    
    // Verify point is on line segment
    // Can compute t and verify 0 <= t <= 1
    let (x1, y1) = (x1, y1);
    let (x2, y2) = (x2, y2);
    
    let dx = x2 - x1;
    let dy = y2 - y1;
    
    if dx.abs() > 1e-6 {
        let t = (px - x1) / dx;
        prop_assert!(t >= -1e-6 && t <= 1.0 + 1e-6);
    }
    
    Ok(())
}
```

---

### Phase 11: Angle Constraints (Unit Circle Method)

**Deliverables:**
- AngleBetweenLinesConstraint
- Unit vector helper functions
- Dot/Cross product in Z3

**Implementation:**

#### src/constraints/angle.rs
```rust
pub struct AngleBetweenLinesConstraint {
    pub line1: LineId,
    pub line2: LineId,
    pub angle: Angle,
}

impl Constraint for AngleBetweenLinesConstraint {
    fn apply<'ctx>(&self, sketch: &Sketch<'ctx>) -> Result<()> {
        let l1 = sketch.get_line(self.line1)
            .ok_or_else(|| TextCADError::InvalidEntity("line1".into()))?;
        let l2 = sketch.get_line(self.line2)
            .ok_or_else(|| TextCADError::InvalidEntity("line2".into()))?;
        
        let l1_start = sketch.get_point(l1.start).unwrap();
        let l1_end = sketch.get_point(l1.end).unwrap();
        let l2_start = sketch.get_point(l2.start).unwrap();
        let l2_end = sketch.get_point(l2.end).unwrap();
        
        // Direction vectors
        let dx1 = l1_end.x.sub(&l1_start.x);
        let dy1 = l1_end.y.sub(&l1_start.y);
        let dx2 = l2_end.x.sub(&l2_start.x);
        let dy2 = l2_end.y.sub(&l2_start.y);
        
        // Normalize to unit vectors
        // len1 = sqrt(dx1² + dy1²)
        let len1_sq = dx1.mul(&dx1).add(&dy1.mul(&dy1));
        let len2_sq = dx2.mul(&dx2).add(&dy2.mul(&dy2));
        
        // Unit vectors: u1 = (dx1/len1, dy1/len1)
        // Instead, use: u1 * len1 = (dx1, dy1)
        // And: u1 · u1 = 1
        
        let u1x = Real::new_const(sketch.context(), "u1x");
        let u1y = Real::new_const(sketch.context(), "u1y");
        let u2x = Real::new_const(sketch.context(), "u2x");
        let u2y = Real::new_const(sketch.context(), "u2y");
        
        // u1 is unit: u1x² + u1y² = 1
        let one = Real::from_real(sketch.context(), 1, 1);
        sketch.solver.assert(
            &u1x.mul(&u1x).add(&u1y.mul(&u1y))._eq(&one)
        );
        sketch.solver.assert(
            &u2x.mul(&u2x).add(&u2y.mul(&u2y))._eq(&one)
        );
        
        // u1 parallel to direction: u1 × (dx1, dy1) = 0
        // u1x * dy1 - u1y * dx1 = k * sqrt(len1_sq) for some scale k
        // Simpler: (u1x, u1y) = ±(dx1, dy1) / len1
        // So: u1x * len1 = ±dx1, u1y * len1 = ±dy1
        
        // Let's use: u1x * sqrt(len1_sq) = dx1
        // This requires sqrt which Z3 doesn't have directly
        // Alternative: introduce len1, len2 as variables with len^2 = len_sq
        
        let len1 = Real::new_const(sketch.context(), "len1");
        let len2 = Real::new_const(sketch.context(), "len2");
        
        sketch.solver.assert(&len1.mul(&len1)._eq(&len1_sq));
        sketch.solver.assert(&len2.mul(&len2)._eq(&len2_sq));
        
        // Now: u1 * len1 = (dx1, dy1)
        sketch.solver.assert(&u1x.mul(&len1)._eq(&dx1));
        sketch.solver.assert(&u1y.mul(&len1)._eq(&dy1));
        sketch.solver.assert(&u2x.mul(&len2)._eq(&dx2));
        sketch.solver.assert(&u2y.mul(&len2)._eq(&dy2));
        
        // Dot product: u1 · u2 = cos(angle)
        let dot = u1x.mul(&u2x).add(&u1y.mul(&u2y));
        
        let cos_val = self.angle.as_radians().cos();
        let cos_rational = Real::from_real(sketch.context(),
            (cos_val * 1_000_000.0) as i64, 1_000_000);
        
        sketch.solver.assert(&dot._eq(&cos_rational));
        
        Ok(())
    }
}
```

**Tests:**
- [ ] Right angle (90°) between perpendicular lines
- [ ] 45° angle
- [ ] 60° angle
- [ ] 0° angle (parallel lines)

**Property-Based Test:**
```rust
#[proptest]
fn prop_angle_symmetry(angle_deg: f64) {
    let angle_deg = angle_deg.abs() % 180.0;
    
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    // Create two lines with specified angle
    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);
    let p3 = sketch.add_point(None);
    let p4 = sketch.add_point(None);
    
    sketch.add_constraint(PointAtPosition {
        point: p1,
        x: Length::meters(0.0),
        y: Length::meters(0.0),
    });
    sketch.add_constraint(PointAtPosition {
        point: p2,
        x: Length::meters(1.0),
        y: Length::meters(0.0),
    });
    
    let line1 = sketch.add_line(p1, p2, None);
    let line2 = sketch.add_line(p3, p4, None);
    
    sketch.add_constraint(AngleBetweenLinesConstraint {
        line1,
        line2,
        angle: Angle::degrees(angle_deg),
    });
    
    // TODO: Verify angle is correct in solution
    let solution = sketch.solve();
    prop_assert!(solution.is_ok());
}
```

---

### Phase 12: SVG Export - Basics

**Deliverables:**
- Export trait
- SVGExporter with fixed defaults
- Coordinate transformation (meters → SVG units)

**Implementation:**

#### src/export/mod.rs
```rust
pub trait Exporter {
    fn export(&self, sketch: &Sketch, solution: &Solution) -> Result<String>;
}
```

#### src/export/svg.rs
```rust
pub struct SVGExporter {
    scale: f64, // meters to SVG units (default: 1m = 1000 units)
    stroke_width: f64,
    view_box_padding: f64,
}

impl Default for SVGExporter {
    fn default() -> Self {
        Self {
            scale: 1000.0, // 1 meter = 1000 SVG units (mm)
            stroke_width: 2.0,
            view_box_padding: 10.0,
        }
    }
}

impl SVGExporter {
    pub fn new() -> Self {
        Self::default()
    }
    
    fn to_svg_coords(&self, x: f64, y: f64) -> (f64, f64) {
        (x * self.scale, -y * self.scale) // Flip Y for SVG
    }
}

impl Exporter for SVGExporter {
    fn export(&self, sketch: &Sketch, solution: &Solution) -> Result<String> {
        let mut svg = String::new();
        
        // Calculate bounding box
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        
        for (_id, point) in &solution.point_coords {
            let (x, y) = self.to_svg_coords(point.0, point.1);
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        
        let width = max_x - min_x + 2.0 * self.view_box_padding;
        let height = max_y - min_y + 2.0 * self.view_box_padding;
        
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">"#,
            min_x - self.view_box_padding,
            min_y - self.view_box_padding,
            width,
            height
        ));
        svg.push('\n');
        
        // Export lines
        for (_, line) in sketch.lines.iter() {
            let p1 = solution.point_coords.get(&line.start).unwrap();
            let p2 = solution.point_coords.get(&line.end).unwrap();
            
            let (x1, y1) = self.to_svg_coords(p1.0, p1.1);
            let (x2, y2) = self.to_svg_coords(p2.0, p2.1);
            
            svg.push_str(&format!(
                r#"  <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="black" stroke-width="{}"/>"#,
                x1, y1, x2, y2, self.stroke_width
            ));
            svg.push('\n');
        }
        
        // Export circles
        for (_, circle) in sketch.circles.iter() {
            let center = solution.point_coords.get(&circle.center).unwrap();
            let (cx, cy) = self.to_svg_coords(center.0, center.1);
            
            // Extract radius from solution
            let radius_meters = solution.model.eval(&circle.radius, true)
                .and_then(|r| r.as_real())
                .map(|(n, d)| n as f64 / d as f64)
                .unwrap_or(1.0);
            let radius_svg = radius_meters * self.scale;
            
            svg.push_str(&format!(
                r#"  <circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="none" stroke="black" stroke-width="{}"/>"#,
                cx, cy, radius_svg, self.stroke_width
            ));
            svg.push('\n');
        }
        
        svg.push_str("</svg>\n");
        
        Ok(svg)
    }
}
```

**Tests:**
- [ ] Export sketch with single line to valid SVG
- [ ] Export sketch with circle to valid SVG
- [ ] SVG can be opened in browser (smoke test)
- [ ] Coordinate transformation is correct

**Integration Test:**
```rust
#[test]
fn test_svg_export_simple_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);
    
    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);
    
    sketch.add_constraint(PointAtPosition {
        point: p1,
        x: Length::meters(0.0),
        y: Length::meters(0.0),
    });
    sketch.add_constraint(PointAtPosition {
        point: p2,
        x: Length::meters(0.1), // 10cm
        y: Length::meters(0.1),
    });
    
    let line = sketch.add_line(p1, p2, None);
    
    let solution = sketch.solve().unwrap();
    
    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).unwrap();
    
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<line"));
    assert!(svg.contains("</svg>"));
    
    // Write to file for manual inspection
    std::fs::write("test_output.svg", svg).unwrap();
}
```

---

## Later Phases (Deferred)

### Phase 13: View System (REQ-6-8)
- Native view definition
- Custom views with rotations
- View-based spatial queries
- View-based constraints

### Phase 14: Fluent API
- Builder pattern for entities
- Chainable constraint specification
- Method chaining syntax

### Phase 15: Bézier Curves
- Bézier curve entity
- Control points
- Point on curve constraint
- Tangent constraints

### Phase 16: Complex Constraints
- Tangential constraint
- Concentric circles
- Equal length/radius
- Symmetry constraints

### Phase 17: Auxiliary Types
- Vector as first-class entity
- Ray entity
- Planar region entity
- Angle as first-class entity

---

## Development Workflow

### Local Development with Nix
```bash
# Enter development environment
nix develop

# Build project
cargo build

# Run tests
cargo test

# Run with optimizations
cargo build --release

# Format code
cargo fmt

# Lint
cargo clippy
```

### With direnv (Optional)
```bash
# One-time setup
echo "use flake" > .envrc
direnv allow

# Then automatically activated when entering directory
cd textcad/
cargo build  # Z3 is automatically available
```

### CI/CD
GitHub Actions automatically:
- Builds on every push/PR
- Runs all tests
- Checks formatting
- Runs clippy lints
- Caches Nix store and Cargo artifacts

---

## Testing Recommendations

### Property-Based Testing Crate
Use **`proptest`** for modern shrinking features.

Add to `Cargo.toml`:
```toml
[dev-dependencies]
proptest = "1.4"
```

### Coverage
Consider adding code coverage with `cargo-tarpaulin`:
```bash
nix develop --command cargo tarpaulin --out Html
```

---

## Success Criteria

Each phase is complete when:
1. All unit tests pass
2. All integration tests pass
3. Property-based tests pass (where applicable)
4. Code is formatted (`cargo fmt`)
5. No clippy warnings
6. Documentation comments added for public APIs
7. Phase deliverables are fully functional

---

## Summary

This implementation plan provides a structured, incremental approach to building TextCAD. Each phase builds on previous work and is independently testable. The plan prioritizes:

1. **Infrastructure first** - Reproducible builds with Nix
2. **Bottom-up construction** - Simple entities before complex ones
3. **Test coverage** - Multiple testing strategies at each phase
4. **Iterative refinement** - Complex features deferred to later phases

The architecture maintains flexibility for future extensions while delivering a functional 2D CAD system with constraint solving and SVG export.
