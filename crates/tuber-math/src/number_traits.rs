pub trait One {
    fn one() -> Self;
}

impl One for i32 {
    fn one() -> Self {
        1
    }
}

impl One for f32 {
    fn one() -> Self {
        1.0
    }
}

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for i32 {
    fn zero() -> Self {
        0
    }
}

impl Zero for f32 {
    fn zero() -> Self {
        0.0
    }
}

pub trait Pi {
    fn pi() -> Self;
}

impl Pi for f32 {
    fn pi() -> Self {
        std::f32::consts::PI
    }
}

impl Pi for f64 {
    fn pi() -> Self {
        std::f64::consts::PI
    }
}
