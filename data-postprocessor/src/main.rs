use std::{env, fs, io::{Write, Read}};

fn main() {
	let args: Vec<_> = env::args().collect();
	if args.len() < 2 {
		println!("Please spesify a filename");
		return;
	}
	let input_file = args[1].clone();
	let output_file: String;
	if args.len() > 2 {
		output_file = args[2].clone();
	} else {
		output_file = input_file.clone() + ".sorted";
	}
	println!("Reading {}...", &input_file);
	let mut data = get_file_timestamps(&input_file).expect("Failed to read file");
	println!("Sorting {} packet timestamps...", data.len());
	data.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
	println!(
		"Sorted packets successfully. Writing to {}...",
		&output_file
	);
	write_file_timestamps(&output_file, &data).expect("Failed to write data");
	println!("Done");
}

fn get_file_timestamps(filename: &String) -> Option<Vec<[f64; 2]>> {
	let mut data: Vec<[f64; 2]> = vec![];
	let mut file = fs::File::open(filename).ok()?;
	let mut buffer: [u8; 8] = [0; 8];
	loop {
		let bytes_read = std::io::Read::by_ref(&mut file).take(8).read(&mut buffer).ok()?;
		if bytes_read < 8 {
			break;
		}
		let transmit_time = f64::from_ne_bytes(buffer);
		let bytes_read = std::io::Read::by_ref(&mut file).take(8).read(&mut buffer).ok()?;
		if bytes_read < 8 {
			break;
		}
		let arrival_time = f64::from_ne_bytes(buffer);
		data.push([transmit_time, arrival_time]);
	}
	Some(data)
}

fn write_file_timestamps(filename: &String, data: &Vec<[f64; 2]>) -> Option<()> {
	let mut file = fs::File::create(&filename).ok()?;
	for pair in data {
		file.write_all(&pair[0].to_ne_bytes()).ok();
		file.write_all(&pair[1].to_ne_bytes()).ok();
	}
	Some(())
}
