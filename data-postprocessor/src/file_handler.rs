use std::{
	fs::File,
	io::{Read, Seek},
};

const CHUNK_SIZE: usize = 512 * 512;
pub struct TrexDataFile {
	file: File,
	buffer: [u8; CHUNK_SIZE],
	buffer_index: usize,
	index_max: usize,
	points_read: usize,
	total_data_points: usize,
}

impl TrexDataFile {
	pub fn new(filename: &str) -> std::io::Result<Self> {
		let file = File::open(filename)?;
		let file_size = file.metadata()?.len();
		Ok(Self {
			file,
			buffer: [0; CHUNK_SIZE],
			buffer_index: CHUNK_SIZE,
			index_max: CHUNK_SIZE,
			points_read: 0,
			total_data_points: file_size as usize / 16,
		})
	}

	pub fn reset(&mut self) -> std::io::Result<()> {
		self.file.seek(std::io::SeekFrom::Start(0))?;
		self.buffer_index = CHUNK_SIZE;
		self.index_max = CHUNK_SIZE;
		self.points_read = 0;
		Ok(())
	}

	pub fn get_last_point(&mut self) -> Option<(f64, f64)> {
		let old_pos = self.file.stream_position().ok()?;
		self.file.seek(std::io::SeekFrom::End(-16)).ok()?;
		let mut temp_buffer: [u8; 16] = [0; 16];
		self.file
			.by_ref()
			.take(16)
			.read_exact(&mut temp_buffer)
			.ok()?;
		let transmit_time = f64::from_le_bytes(temp_buffer[0..8].try_into().unwrap());
		let arrival_time = f64::from_le_bytes(temp_buffer[8..16].try_into().unwrap());
		self.file.seek(std::io::SeekFrom::Start(old_pos)).ok()?;
		Some((transmit_time, arrival_time))
	}
}

impl Iterator for TrexDataFile {
	type Item = (f64, f64);

	fn next(&mut self) -> Option<Self::Item> {
		if self.buffer_index >= self.index_max {
			self.index_max = std::io::Read::by_ref(&mut self.file)
				.take(CHUNK_SIZE.try_into().unwrap())
				.read(&mut self.buffer)
				.ok()?;
			if self.index_max < 16 {
				return None;
			}
			self.buffer_index = 0;
		}
		let transmit_time = f64::from_le_bytes(
			self.buffer[self.buffer_index..self.buffer_index + 8]
				.try_into()
				.unwrap(),
		);
		let arrival_time = f64::from_le_bytes(
			self.buffer[self.buffer_index + 8..self.buffer_index + 16]
				.try_into()
				.unwrap(),
		);
		self.buffer_index += 16;
		self.points_read += 1;
		return Some((transmit_time, arrival_time));
	}
}

impl ExactSizeIterator for TrexDataFile {
	fn len(&self) -> usize {
		self.total_data_points - self.points_read
	}
}
