use std::{
	env, fs,
	io::{Read, Write},
	vec,
};

use plotters::prelude::*;

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
	let filename;
	if args.len() > 3 && args.last().unwrap().ends_with(".png") {
		filename = args.last().unwrap().clone();
	} else {
		filename = args[2].clone() + ".png";
	}
	let highest_latency = data
		.arrival_times
		.iter()
		.reduce(|a, b| if a > b { a } else { b })
		.unwrap()
		.clone();
	let lowest_latency = data
		.arrival_times
		.iter()
		.reduce(|a, b| if a < b { a } else { b })
		.unwrap()
		.clone();
	println!(
		"Packets range from {} to {}",
		&lowest_latency, &highest_latency
	);
	let root = BitMapBackend::new(&filename, (2400, 1600)).into_drawing_area();
	root.fill(&BLACK).unwrap();
	let mut chart = ChartBuilder::on(&root)
		.set_label_area_size(LabelAreaPosition::Left, 160)
		.set_label_area_size(LabelAreaPosition::Bottom, 100)
		.caption("Latencies", ("sans-serif", 50).into_font().color(&WHITE))
		.margin(10)
		.build_cartesian_2d(
			data.transmit_times[0].round()..data.transmit_times.last().unwrap().round(),
			(0.9 * lowest_latency)..(1.1 * highest_latency),
		)
		.unwrap();
	chart
		.configure_mesh()
		.axis_style(&WHITE)
		.label_style(("sans-serif", 32).into_font().color(&WHITE))
		.x_desc("Transmit time (s)")
		.y_desc("Latency (µs)")
		.draw()
		.unwrap();
	chart
		.draw_series(
			(0..data.transmit_times.len())
				.map(|i| Pixel::new((data.transmit_times[i], data.arrival_times[i]), &GREEN)),
		)
		.unwrap();
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
