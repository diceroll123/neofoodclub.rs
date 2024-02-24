#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Chance {
    pub value: u32,
    pub probability: f64,
    pub cumulative: f64,
    pub tail: f64,
}
