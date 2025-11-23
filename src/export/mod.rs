//! Export functionality for TextCAD sketches
//!
//! This module provides traits and implementations for exporting solved
//! sketches to various file formats.

pub mod svg;

pub use svg::SVGExporter;

use crate::error::Result;
use crate::sketch::Sketch;
use crate::solution::Solution;

/// Trait for exporting sketches to various formats
///
/// Implementors of this trait can convert a solved sketch with its
/// solution into a specific file format (SVG, STL, etc.).
pub trait Exporter {
    /// Export a sketch with its solution to a string representation
    ///
    /// # Arguments
    /// * `sketch` - The sketch containing geometric entities
    /// * `solution` - The solution containing solved coordinates
    ///
    /// # Returns
    /// String representation in the target format
    ///
    /// # Example
    /// ```no_run
    /// use textcad::export::{Exporter, SVGExporter};
    /// # use textcad::{Sketch, Solution};
    /// # let sketch = todo!();
    /// # let solution = todo!();
    ///
    /// let exporter = SVGExporter::new();
    /// let svg_content = exporter.export(&sketch, &solution).unwrap();
    /// ```
    fn export(&self, sketch: &Sketch, solution: &Solution) -> Result<String>;
}
