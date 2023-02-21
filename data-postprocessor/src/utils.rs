use crate::TrexData;

pub fn vector_avg(v: &[f64]) -> f64 {
	v.iter().fold(0.0, |a, b| a + b) / v.len() as f64
}

pub fn vector_max(v: &[f64]) -> f64 {
	v.iter().fold(f64::MIN, |a, b| a.max(*b))
}

pub fn vector_min(v: &[f64]) -> f64 {
	v.iter().fold(f64::MAX, |a, b| a.min(*b))
}

pub fn trexdata_to_latency(data: &mut TrexData) {
	let first_arrival = data.transmit_times[0];
	for i in 0..data.transmit_times.len() {
		// Replace the arrival_times vector with an exact latency value. Store in µs instead of s
		data.arrival_times[i] = (data.arrival_times[i] - data.transmit_times[i]) * 1_000_000.0;
		// transmit_times are relative to when TRex was started, we want them to start at 0 instead
		data.transmit_times[i] -= first_arrival;
	}
}
