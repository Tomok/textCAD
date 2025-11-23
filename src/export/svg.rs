//! SVG export implementation
//!
//! Provides SVG export functionality for TextCAD sketches, converting
//! solved geometric entities into SVG format.

use crate::error::Result;
use crate::export::Exporter;
use crate::sketch::Sketch;
use crate::solution::Solution;

/// SVG exporter with configurable rendering parameters
///
/// SVGExporter converts solved sketches into SVG format with proper
/// coordinate transformations and viewBox calculation.
#[derive(Debug, Clone)]
pub struct SVGExporter {
    /// Scale factor from meters to SVG units (default: 1m = 1000 units)
    scale: f64,
    /// Stroke width for rendered entities in SVG units
    stroke_width: f64,
    /// Padding around the bounding box in SVG units
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
    /// Create a new SVGExporter with default parameters
    ///
    /// Default parameters:
    /// - scale: 1000.0 (1 meter = 1000 SVG units, i.e., millimeters)
    /// - stroke_width: 2.0
    /// - view_box_padding: 10.0
    ///
    /// # Example
    /// ```
    /// use textcad::export::SVGExporter;
    ///
    /// let exporter = SVGExporter::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform coordinates from meters to SVG coordinate system
    ///
    /// This method scales coordinates and flips the Y axis to match
    /// SVG's top-down coordinate system.
    ///
    /// # Arguments
    /// * `x` - X coordinate in meters
    /// * `y` - Y coordinate in meters
    ///
    /// # Returns
    /// Tuple of (x, y) in SVG coordinate system
    fn to_svg_coords(&self, x: f64, y: f64) -> (f64, f64) {
        (x * self.scale, -y * self.scale) // Flip Y for SVG
    }
}

impl Exporter for SVGExporter {
    /// Export a sketch with its solution to SVG format
    ///
    /// This method generates a complete SVG document with proper viewBox,
    /// namespace, and all geometric entities rendered.
    ///
    /// # Arguments
    /// * `sketch` - The sketch containing geometric entities
    /// * `solution` - The solution containing solved coordinates
    ///
    /// # Returns
    /// String containing the complete SVG document
    ///
    /// # Example
    /// ```no_run
    /// use textcad::export::{Exporter, SVGExporter};
    /// # use textcad::{Sketch, Solution};
    /// # let sketch = todo!();
    /// # let solution = todo!();
    ///
    /// let exporter = SVGExporter::new();
    /// let svg = exporter.export(&sketch, &solution).unwrap();
    /// println!("{}", svg);
    /// ```
    fn export(&self, sketch: &Sketch, solution: &Solution) -> Result<String> {
        let mut svg = String::new();

        // Calculate bounding box from all point coordinates
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for (_id, point) in solution.all_point_coordinates() {
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
        for (_, line) in sketch.lines() {
            let p1 = solution.all_point_coordinates().get(&line.start).unwrap();
            let p2 = solution.all_point_coordinates().get(&line.end).unwrap();

            let (x1, y1) = self.to_svg_coords(p1.0, p1.1);
            let (x2, y2) = self.to_svg_coords(p2.0, p2.1);

            svg.push_str(&format!(
                r#"  <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="black" stroke-width="{}"/>"#,
                x1, y1, x2, y2, self.stroke_width
            ));
            svg.push('\n');
        }

        // Export circles
        for (_, circle) in sketch.circles() {
            let center = solution
                .all_point_coordinates()
                .get(&circle.center)
                .unwrap();
            let (cx, cy) = self.to_svg_coords(center.0, center.1);

            // Extract radius from solution
            let radius_meters = solution
                .model()
                .eval(&circle.radius, true)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_exporter_creation() {
        let exporter = SVGExporter::new();
        assert_eq!(exporter.scale, 1000.0);
        assert_eq!(exporter.stroke_width, 2.0);
        assert_eq!(exporter.view_box_padding, 10.0);
    }

    #[test]
    fn test_svg_exporter_default() {
        let exporter = SVGExporter::default();
        assert_eq!(exporter.scale, 1000.0);
        assert_eq!(exporter.stroke_width, 2.0);
        assert_eq!(exporter.view_box_padding, 10.0);
    }

    #[test]
    fn test_coordinate_transformation() {
        let exporter = SVGExporter::new();

        // Test positive coordinates
        let (x, y) = exporter.to_svg_coords(1.0, 2.0);
        assert_eq!(x, 1000.0);
        assert_eq!(y, -2000.0); // Y is flipped

        // Test negative coordinates
        let (x, y) = exporter.to_svg_coords(-0.5, -0.3);
        assert_eq!(x, -500.0);
        assert_eq!(y, 300.0); // Y is flipped

        // Test zero
        let (x, y) = exporter.to_svg_coords(0.0, 0.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_coordinate_transformation_with_custom_scale() {
        let mut exporter = SVGExporter::new();
        exporter.scale = 100.0; // Custom scale: 1m = 100 units (cm)

        // Test with custom scale
        let (x, y) = exporter.to_svg_coords(1.0, 2.0);
        assert_eq!(x, 100.0);
        assert_eq!(y, -200.0); // Y is flipped

        // Test fractional values
        let (x, y) = exporter.to_svg_coords(0.5, -0.25);
        assert_eq!(x, 50.0);
        assert_eq!(y, 25.0); // Y is flipped
    }

    #[test]
    fn test_y_axis_flip_property() {
        let exporter = SVGExporter::new();

        // Test that Y-axis flip is consistent
        let y_values = vec![1.0, -1.0, 0.0, 100.0, -100.0, 0.001, -0.001];

        for y in y_values {
            let (_, svg_y) = exporter.to_svg_coords(0.0, y);
            // SVG y should be the negative of input y (scaled)
            assert_eq!(svg_y, -y * exporter.scale);
        }
    }

    #[test]
    fn test_coordinate_transformation_symmetry() {
        let exporter = SVGExporter::new();

        // Test that mirrored coordinates produce mirrored results
        let (x1, y1) = exporter.to_svg_coords(5.0, 3.0);
        let (x2, y2) = exporter.to_svg_coords(-5.0, -3.0);

        assert_eq!(x1, -x2);
        assert_eq!(y1, -y2);
    }

    #[test]
    fn test_default_parameters() {
        let exporter = SVGExporter::default();

        // Verify default parameters match documentation
        assert_eq!(
            exporter.scale, 1000.0,
            "Default scale should be 1000.0 (1m = 1000 SVG units)"
        );
        assert_eq!(
            exporter.stroke_width, 2.0,
            "Default stroke width should be 2.0"
        );
        assert_eq!(
            exporter.view_box_padding, 10.0,
            "Default view box padding should be 10.0"
        );
    }

    #[test]
    fn test_exporter_clone() {
        let exporter1 = SVGExporter::new();
        let exporter2 = exporter1.clone();

        assert_eq!(exporter1.scale, exporter2.scale);
        assert_eq!(exporter1.stroke_width, exporter2.stroke_width);
        assert_eq!(exporter1.view_box_padding, exporter2.view_box_padding);
    }

    #[test]
    fn test_exporter_debug() {
        let exporter = SVGExporter::new();
        let debug_str = format!("{:?}", exporter);

        // Should contain the struct name and fields
        assert!(debug_str.contains("SVGExporter"));
        assert!(debug_str.contains("scale"));
        assert!(debug_str.contains("stroke_width"));
        assert!(debug_str.contains("view_box_padding"));
    }
}
