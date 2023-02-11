use std::{
	env, fs,
	io::{Read, Write},
	vec,
};

use plotly::{layout::Axis, Layout, Plot, Scatter};

struct TrexData {
	pub transmit_times: Vec<f64>,
	pub arrival_times: Vec<f64>,
}

fn main() {
	let args: Vec<_> = env::args().collect();
	if args.len() < 3 {
		println!("Please spesify a mode and a filename");
		println!("Valid modes: sort, plot");
		return;
	}
	let input_file = args[2].clone();
	println!("Reading {}...", &input_file);
	let mut data =
		get_file_timestamps(&input_file).expect(&format!("Failed to read file {}", &input_file));
	println!("Read {} packet data points", data.transmit_times.len());
	match args[1].as_str() {
		"sort" => {
			mode_sort_data(&mut data, &args);
		}
		"plot" => {
			mode_plot_data(data, &args);
		}
		_ => {
			println!("Invalid mode option");
		}
	}
}

fn mode_plot_data(mut data: TrexData, args: &Vec<String>) {
	let first_arrival = data.transmit_times[0];
	for i in 0..data.transmit_times.len() {
		data.arrival_times[i] = (data.arrival_times[i] - data.transmit_times[i]) * 1_000_000.0;
		data.transmit_times[i] -= first_arrival;
	}
	let mut plot = Plot::new();
	let trace =
		Scatter::new(data.transmit_times, data.arrival_times).mode(plotly::common::Mode::Markers);
	plot.add_trace(trace);
	plot.set_layout(
		Layout::new()
			.title("<b>Packet latency</b>".into())
			.height(800)
			.width(800)
			.x_axis(Axis::new().title("Arrival time (s)".into()))
			.y_axis(Axis::new().title("Latency (µs)".into())),
	);
	if args.len() > 3 && args.last().unwrap().ends_with(".png") {
		plot.write_image(args.last().unwrap(), plotly::ImageFormat::PNG, 1600, 1600, 1.5);
	} else {
		plot.show();
	}
}

fn mode_sort_data(data: &mut TrexData, args: &Vec<String>) {
	let output_file: String;
	if args.len() > 3 {
		output_file = args[3].clone();
	} else {
		output_file = args[2].clone() + ".sorted";
	}
	println!("Sorting {} packet timestamps...", data.transmit_times.len());
	// data.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
	println!(
		"Sorted packets successfully. Writing to {}...",
		&output_file
	);
	write_file_timestamps(&output_file, &data).expect("Failed to write data");
	println!("Done");
}

fn get_file_timestamps(filename: &String) -> Option<TrexData> {
	let mut data: TrexData = TrexData {
		transmit_times: vec![],
		arrival_times: vec![],
	};
	let mut file = fs::File::open(filename).ok()?;
	const CHUNK_SIZE: usize = 512 * 512;
	let mut buffer: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
	loop {
		let bytes_left = std::io::Read::by_ref(&mut file)
			.take(CHUNK_SIZE.try_into().unwrap())
			.read(&mut buffer)
			.ok()?;
		if bytes_left < 16 {
			break;
		}
		for i in (0..bytes_left).step_by(16) {
			let transmit_time = f64::from_ne_bytes(buffer[i..i + 8].try_into().unwrap());
			let arrival_time = f64::from_ne_bytes(buffer[i + 8..i + 16].try_into().unwrap());
			data.transmit_times.push(transmit_time);
			data.arrival_times.push(arrival_time);
		}
	}
	Some(data)
}

fn write_file_timestamps(filename: &String, data: &TrexData) -> Option<()> {
	let mut file = fs::File::create(&filename).ok()?;
	for i in 0..data.transmit_times.len() {
		file.write_all(&data.transmit_times[i].to_ne_bytes()).ok()?;
		file.write_all(&data.arrival_times[i].to_ne_bytes()).ok()?;
	}
	Some(())
}
