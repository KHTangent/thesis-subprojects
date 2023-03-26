pub fn vector_avg(v: &[f64]) -> f64 {
	v.iter().fold(0.0, |a, b| a + b) / v.len() as f64
}

pub fn vector_max(v: &[f64]) -> f64 {
	v.iter().fold(f64::MIN, |a, b| a.max(*b))
}

pub fn vector_min(v: &[f64]) -> f64 {
	v.iter().fold(f64::MAX, |a, b| a.min(*b))
}
