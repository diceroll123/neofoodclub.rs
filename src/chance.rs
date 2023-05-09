#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Chance {
    pub(crate) value: u32,
    pub(crate) probability: f64,
    pub(crate) cumulative: f64,
    pub(crate) tail: f64,
}
