pub const BASIS_POINT_MAX: u128 = 10000;
pub const PRECISION: u128 = 1_000_000_000_000;
pub const MAX_BINS_PER_POSITION: i32 = 500;
pub const ALLOWED_PARAMETERS: &[(u16, u16)] = &[
    (1, 10),
    (5, 10),
    (20, 10),
    (20, 50),
    (50, 20),
    (50, 100),
    (100, 30),
    (100, 250),
    (200, 500),
];