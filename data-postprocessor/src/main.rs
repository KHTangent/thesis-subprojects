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

enum PlotMode {
	Latency,
	Jitter,
}

fn main() {
	let args: Vec<_> = env::args().collect();
	if args.len() < 3 {
		println!("Please spesify a mode and a filename");
		println!("Valid modes: sort, plot, plotj");
		println!("Sort is currently broken");
		println!("Plot usage: plot/plotj inputfile [c(seconds)] output.png");
		println!("  plot is used to plot packet latencies, plotj for inter-packet times");
		println!("  Use c0.5 to cut 0.5 seconds of each side of the plot, to account for warmup/cooldown");
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
			mode_plot_data(data, &args, PlotMode::Latency);
		}
		"plotj" => mode_plot_data(data, &args, PlotMode::Jitter),
		_ => {
			println!("Invalid mode option");
		}
	}
}

fn mode_plot_data(mut data: TrexData, args: &Vec<String>, mode: PlotMode) {
	let first_arrival = data.transmit_times[0];
	for i in 0..data.transmit_times.len() {
		// Replace the arrival_times vector with an exact latency value. Store in µs instead of s
		data.arrival_times[i] = (data.arrival_times[i] - data.transmit_times[i]) * 1_000_000.0;
		// transmit_times are relative to when TRex was started, we want them to start at 0 instead
		data.transmit_times[i] -= first_arrival;
	}
	let output_file;
	if args.len() > 3 && args.last().unwrap().ends_with(".png") {
		output_file = args.last().unwrap().clone();
	} else {
		output_file = args[2].clone() + ".png";
	}
	// Used for cutting off warmup-time and cooldown-time
	let start_at;
	let end_at;
	if args.len() > 3 && args[3].starts_with('c') {
		let a = args[3][1..].parse::<f64>().unwrap_or(0.0);
		start_at =
			(data.transmit_times.len() as f64 * a / data.transmit_times.last().unwrap()) as usize;
		end_at = data.transmit_times.len() - start_at;
	} else {
		start_at = 0;
		end_at = data.transmit_times.len();
	}
	// Used for creating good values for the axises
	let mut highest_latency = data.arrival_times[start_at..end_at]
		.iter()
		.fold(f64::MIN, |a, b| a.max(*b));
	let mut lowest_latency = data.arrival_times[start_at..end_at]
		.iter()
		.fold(f64::MAX, |a, b| a.min(*b));
	// Used for printing stats
	let average_latency: f64 = data.arrival_times[start_at..end_at]
		.iter()
		.fold(0.0, |a, b| a + b)
		/ data.arrival_times.len() as f64;
	let variance = data.arrival_times[start_at..end_at]
		.iter()
		.fold(0.0, |a, b| {
			&a + (b - average_latency) * (b - average_latency)
		}) / data.arrival_times.len() as f64;
	let standard_deviation = variance.sqrt();
	println!(
		"Packets range from {} to {} µs, with an average of {} µs",
		&lowest_latency, &highest_latency, &average_latency
	);
	println!("Standard deviation is {} µs", &standard_deviation);

	if let PlotMode::Jitter = mode {
		// Map all values to inter-packet times
		let mut temp: f64;
		let mut prev: f64 = data.arrival_times[0];
		for i in 1..data.arrival_times.len() {
			temp = data.arrival_times[i].clone();
			data.arrival_times[i] -= prev;
			prev = temp;
		}
		data.arrival_times[0] = 0.0;
		highest_latency = data.arrival_times[start_at..end_at]
			.iter()
			.fold(f64::MIN, |a, b| a.max(*b));
		lowest_latency = data.arrival_times[start_at..end_at]
			.iter()
			.fold(f64::MAX, |a, b| a.min(*b));
	}

	let root = BitMapBackend::new(&output_file, (2400, 1600)).into_drawing_area();
	root.fill(&BLACK).unwrap();
	let mut chart = ChartBuilder::on(&root)
		.set_label_area_size(LabelAreaPosition::Left, 160)
		.set_label_area_size(LabelAreaPosition::Bottom, 100)
		.caption(
			match mode {
				PlotMode::Latency => "Latencies",
				PlotMode::Jitter => "Inter-packet times",
			},
			("sans-serif", 50).into_font().color(&WHITE),
		)
		.margin(30)
		.build_cartesian_2d(
			// Calculate time bounds for x-axis as whole values
			data.transmit_times[start_at].round() - 1.0
				..data.transmit_times[end_at - 1].round() + 1.0,
			// Expand the range of the y axis a bit
			(0.9 * lowest_latency)..(1.1 * highest_latency),
		)
		.unwrap();
	chart
		.configure_mesh()
		.axis_style(&WHITE)
		.label_style(("sans-serif", 32).into_font().color(&WHITE))
		.x_desc("Transmit time (s)")
		.y_desc(match mode {
			PlotMode::Latency => "Latency (µs)",
			PlotMode::Jitter => "Inter-packet time (µs)",
		})
		.draw()
		.unwrap();
	chart
		.draw_series(
			(start_at..end_at)
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
			let transmit_time = f64::from_le_bytes(buffer[i..i + 8].try_into().unwrap());
			let arrival_time = f64::from_le_bytes(buffer[i + 8..i + 16].try_into().unwrap());
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
