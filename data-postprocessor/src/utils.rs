pub struct Tally {
	pub count: usize,
	pub sum: f64,
	pub ssum: f64,
	pub min: f64,
	pub max: f64,
}

impl Tally {
	pub fn new() -> Tally {
		Tally {
			count: 0,
			sum: 0.0,
			ssum: 0.0,
			min: f64::MAX,
			max: f64::MIN,
		}
	}

	pub fn add(&mut self, v: f64) {
		self.count += 1;
		self.sum += v;
		self.ssum += v * v;
		self.min = self.min.min(v);
		self.max = self.max.max(v);
	}

	pub fn avg(&self) -> f64 {
		self.sum / self.count as f64
	}

	pub fn stddev(&self) -> f64 {
		let avg = self.avg();
		(self.ssum / self.count as f64 - avg * avg).sqrt()
	}
}
