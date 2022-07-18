use num_traits::Float;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Vector3D<T: Float> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Float> Vector3D<T> {
    pub fn new(x: T, y: T, z: T) -> Vector3D<T> {
        Vector3D { x, y, z }
    }
}

impl<T: Float> Mul<T> for Vector3D<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vector3D<f64>> for f64 {
    type Output = Vector3D<f64>;
    fn mul(self, rhs: Vector3D<f64>) -> Self::Output {
        Vector3D {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}

impl Mul<Vector3D<f32>> for f32 {
    type Output = Vector3D<f32>;
    fn mul(self, rhs: Vector3D<f32>) -> Self::Output {
        Vector3D {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}

impl<T: Float> Add for Vector3D<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: Float> Sub for Vector3D<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> Display for Vector3D<T>
where
    T: Float + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}\ny: {}\nz: {}", self.x, self.y, self.z)
    }
}
