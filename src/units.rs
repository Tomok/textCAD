use std::ops::{Add, Sub, Mul, Div, Neg};

/// A length value stored in meters.
/// Provides type-safe unit conversions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length {
    meters: f64,
}

impl Length {
    /// Create a length from meters
    pub fn meters(value: f64) -> Self {
        Self { meters: value }
    }

    /// Create a length from millimeters
    pub fn millimeters(value: f64) -> Self {
        Self { meters: value / 1000.0 }
    }

    /// Create a length from centimeters
    pub fn centimeters(value: f64) -> Self {
        Self { meters: value / 100.0 }
    }

    /// Create a length from inches
    pub fn inches(value: f64) -> Self {
        Self { meters: value * 0.0254 }
    }

    /// Get the value in meters
    pub fn to_meters(self) -> f64 {
        self.meters
    }

    /// Get the value in millimeters
    pub fn to_millimeters(self) -> f64 {
        self.meters * 1000.0
    }

    /// Get the value in centimeters
    pub fn to_centimeters(self) -> f64 {
        self.meters * 100.0
    }

    /// Get the value in inches
    pub fn to_inches(self) -> f64 {
        self.meters / 0.0254
    }

    /// Check if the length is approximately zero
    pub fn is_zero(self, epsilon: f64) -> bool {
        self.meters.abs() < epsilon
    }
}

impl Add for Length {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self { meters: self.meters + other.meters }
    }
}

impl Sub for Length {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self { meters: self.meters - other.meters }
    }
}

impl Mul<f64> for Length {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self { meters: self.meters * scalar }
    }
}

impl Mul<Length> for f64 {
    type Output = Length;

    fn mul(self, length: Length) -> Self::Output {
        Length { meters: self * length.meters }
    }
}

impl Mul<Length> for Length {
    type Output = Area;

    fn mul(self, other: Length) -> Self::Output {
        Area { square_meters: self.meters * other.meters }
    }
}

impl Div<f64> for Length {
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        Self { meters: self.meters / scalar }
    }
}

impl Div<Length> for Length {
    type Output = f64;

    fn div(self, other: Length) -> Self::Output {
        self.meters / other.meters
    }
}

impl Neg for Length {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { meters: -self.meters }
    }
}

/// An area value stored in square meters.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Area {
    square_meters: f64,
}

impl Area {
    /// Create an area from square meters
    pub fn square_meters(value: f64) -> Self {
        Self { square_meters: value }
    }

    /// Get the value in square meters
    pub fn to_square_meters(self) -> f64 {
        self.square_meters
    }

    /// Get the value in square millimeters
    pub fn to_square_millimeters(self) -> f64 {
        self.square_meters * 1_000_000.0
    }
}

impl Add for Area {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self { square_meters: self.square_meters + other.square_meters }
    }
}

impl Sub for Area {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self { square_meters: self.square_meters - other.square_meters }
    }
}

impl Mul<f64> for Area {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self { square_meters: self.square_meters * scalar }
    }
}

impl Mul<Area> for f64 {
    type Output = Area;

    fn mul(self, area: Area) -> Self::Output {
        Area { square_meters: self * area.square_meters }
    }
}

impl Div<f64> for Area {
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        Self { square_meters: self.square_meters / scalar }
    }
}

impl Div<Length> for Area {
    type Output = Length;

    fn div(self, length: Length) -> Self::Output {
        Length { meters: self.square_meters / length.meters }
    }
}

impl Div<Area> for Area {
    type Output = f64;

    fn div(self, other: Area) -> Self::Output {
        self.square_meters / other.square_meters
    }
}

/// An angle value stored in radians.
/// Provides type-safe unit conversions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Angle {
    radians: f64,
}

impl Angle {
    /// Create an angle from radians
    pub fn radians(value: f64) -> Self {
        Self { radians: value }
    }

    /// Create an angle from degrees
    pub fn degrees(value: f64) -> Self {
        Self { radians: value * std::f64::consts::PI / 180.0 }
    }

    /// Get the value in radians
    pub fn to_radians(self) -> f64 {
        self.radians
    }

    /// Get the value in degrees
    pub fn to_degrees(self) -> f64 {
        self.radians * 180.0 / std::f64::consts::PI
    }

    /// Normalize angle to [0, 2π)
    pub fn normalize(self) -> Self {
        let mut rad = self.radians % (2.0 * std::f64::consts::PI);
        if rad < 0.0 {
            rad += 2.0 * std::f64::consts::PI;
        }
        Self { radians: rad }
    }

    /// Normalize angle to [-π, π)
    pub fn normalize_symmetric(self) -> Self {
        let mut rad = self.radians % (2.0 * std::f64::consts::PI);
        if rad >= std::f64::consts::PI {
            rad -= 2.0 * std::f64::consts::PI;
        } else if rad < -std::f64::consts::PI {
            rad += 2.0 * std::f64::consts::PI;
        }
        Self { radians: rad }
    }

    /// Compute sine of the angle
    pub fn sin(self) -> f64 {
        self.radians.sin()
    }

    /// Compute cosine of the angle
    pub fn cos(self) -> f64 {
        self.radians.cos()
    }

    /// Compute tangent of the angle
    pub fn tan(self) -> f64 {
        self.radians.tan()
    }

    /// Check if the angle is approximately zero
    pub fn is_zero(self, epsilon: f64) -> bool {
        self.radians.abs() < epsilon
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self { radians: self.radians + other.radians }
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self { radians: self.radians - other.radians }
    }
}

impl Mul<f64> for Angle {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self { radians: self.radians * scalar }
    }
}

impl Mul<Angle> for f64 {
    type Output = Angle;

    fn mul(self, angle: Angle) -> Self::Output {
        Angle { radians: self * angle.radians }
    }
}

impl Div<f64> for Angle {
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        Self { radians: self.radians / scalar }
    }
}

impl Div<Angle> for Angle {
    type Output = f64;

    fn div(self, other: Angle) -> Self::Output {
        self.radians / other.radians
    }
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { radians: -self.radians }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_conversions() {
        let len = Length::meters(1.0);
        assert_eq!(len.to_meters(), 1.0);
        assert_eq!(len.to_millimeters(), 1000.0);
        assert_eq!(len.to_centimeters(), 100.0);
        
        let len_mm = Length::millimeters(1000.0);
        assert_eq!(len_mm.to_meters(), 1.0);
    }

    #[test]
    fn test_length_arithmetic() {
        let a = Length::meters(2.0);
        let b = Length::meters(3.0);
        
        assert_eq!((a + b).to_meters(), 5.0);
        assert_eq!((b - a).to_meters(), 1.0);
        assert_eq!((a * 2.0).to_meters(), 4.0);
        assert_eq!((a / 2.0).to_meters(), 1.0);
        assert_eq!(b / a, 1.5);
    }

    #[test]
    fn test_length_multiplication_creates_area() {
        let a = Length::meters(3.0);
        let b = Length::meters(4.0);
        let area = a * b;
        assert_eq!(area.to_square_meters(), 12.0);
    }

    #[test]
    fn test_area_division_by_length() {
        let area = Area::square_meters(12.0);
        let width = Length::meters(3.0);
        let height = area / width;
        assert_eq!(height.to_meters(), 4.0);
    }

    #[test]
    fn test_angle_conversions() {
        let angle = Angle::degrees(90.0);
        assert!((angle.to_radians() - std::f64::consts::PI / 2.0).abs() < 1e-10);
        
        let angle_rad = Angle::radians(std::f64::consts::PI);
        assert!((angle_rad.to_degrees() - 180.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_normalization() {
        let angle = Angle::degrees(450.0);
        let normalized = angle.normalize();
        assert!((normalized.to_degrees() - 90.0).abs() < 1e-10);
        
        let angle = Angle::degrees(-90.0);
        let normalized = angle.normalize_symmetric();
        assert!((normalized.to_degrees() + 90.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_arithmetic() {
        let a = Angle::degrees(30.0);
        let b = Angle::degrees(60.0);
        
        assert!((a + b).to_degrees() - 90.0 < 1e-10);
        assert!((b - a).to_degrees() - 30.0 < 1e-10);
        assert!((a * 2.0).to_degrees() - 60.0 < 1e-10);
        assert!((b / a) - 2.0 < 1e-10);
    }

    #[test]
    fn test_trig_functions() {
        let angle = Angle::degrees(90.0);
        assert!((angle.sin() - 1.0).abs() < 1e-10);
        assert!(angle.cos().abs() < 1e-10);
        
        let angle = Angle::degrees(45.0);
        assert!((angle.sin() - std::f64::consts::SQRT_2 / 2.0).abs() < 1e-10);
        assert!((angle.cos() - std::f64::consts::SQRT_2 / 2.0).abs() < 1e-10);
    }
}