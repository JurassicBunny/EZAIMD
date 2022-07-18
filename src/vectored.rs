use crate::vector::Vector3D;
use num_traits::Float;
use serde::{Deserialize, Serialize};

pub trait Vectored<T: Float> {
    fn as_vec(&self) -> Vector3D<T>;

    fn new(x: T, y: T, z: T) -> Self;

    fn set_vec(&mut self, vec: Vector3D<T>);

    fn sqr_norm(&self) -> T {
        self.as_vec().x.powi(2) + self.as_vec().y.powi(2) + self.as_vec().z.powi(2)
    }

    fn norm(&self) -> T {
        self.sqr_norm().sqrt()
    }

    fn normalize(mut self) -> Self
    where
        Self: Sized,
    {
        let result = self.as_vec() * self.norm().recip();
        self.set_vec(result);
        self
    }
}

#[macro_export]
macro_rules! make_vectored {
    ($expression:ident) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
        pub struct $expression<T: num_traits::Float>(Vector3D<T>);

        impl<T: num_traits::Float> Vectored<T> for $expression<T> {
            fn new(x: T, y: T, z: T) -> Self {
                let vector = Vector3D::new(x, y, z);
                Self(vector)
            }

            fn as_vec(&self) -> Vector3D<T> {
                self.0
            }
            fn set_vec(&mut self, vec: Vector3D<T>) {
                self.0 = vec;
            }
        }

        impl<T> $expression<T>
        where
            T: num_traits::Float,
        {
            pub fn as_vectype<U>(self) -> U
            where
                U: crate::vectored::Vectored<T>,
            {
                let x = self.as_vec().x;
                let y = self.as_vec().y;
                let z = self.as_vec().z;

                U::new(x, y, z)
            }
        }

        impl<T, U> std::ops::Add<U> for $expression<T>
        where
            T: num_traits::Float,
            U: Vectored<T>,
        {
            type Output = $expression<T>;
            fn add(mut self, rhs: U) -> Self::Output {
                let result = self.as_vec() + rhs.as_vec();
                self.set_vec(result);
                self
            }
        }

        impl<T, U> std::ops::Sub<U> for $expression<T>
        where
            T: num_traits::Float,
            U: crate::vectored::Vectored<T>,
        {
            type Output = $expression<T>;
            fn sub(mut self, rhs: U) -> Self::Output {
                let result = self.as_vec() - rhs.as_vec();
                self.set_vec(result);
                self
            }
        }

        impl<T> std::ops::Mul<T> for $expression<T>
        where
            T: num_traits::Float,
        {
            type Output = $expression<T>;
            fn mul(mut self, rhs: T) -> Self {
                let result = self.as_vec() * rhs;
                self.set_vec(result);
                self
            }
        }

        impl std::ops::Mul<$expression<f64>> for f64 {
            type Output = $expression<f64>;
            fn mul(self, mut rhs: $expression<f64>) -> Self::Output {
                let result = self * rhs.as_vec();
                rhs.set_vec(result);
                rhs
            }
        }

        impl std::ops::Mul<$expression<f32>> for f32 {
            type Output = $expression<f32>;
            fn mul(self, mut rhs: $expression<f32>) -> Self::Output {
                let result = self * rhs.as_vec();
                rhs.set_vec(result);
                rhs
            }
        }
    };
}

make_vectored!(Acceleration);
make_vectored!(Force);
make_vectored!(Momentum);
make_vectored!(Position);
make_vectored!(Velocity);
