use clap::{Parser, Subcommand, ValueEnum};
use plotters::prelude::*;
use std::{fs, io::Read, vec};

use crate::file_handler::TrexDataFile;

mod file_handler;
mod utils;

#[derive(Parser)]
#[command(about)]
struct Cli {
	/// Input file from TRex
	input_file: String,
	/// Mode to do
	#[command(subcommand)]
	mode: Modes,
}

#[derive(Subcommand)]
enum Modes {
	/// Generate plots
	Plot {
		/// What to plot
		#[arg(short, long)]
		plot_mode: PlotMode,
		/// Where to store generated plot
		#[arg(short, long)]
		output_file: String,
		/// Seconds to cut off at the ends of the file
		#[arg(short, long)]
		cut: Option<f64>,
	},
	/// Test the suitability for real-time applications
	Validate {
		/// Treshold (in µs) for a packet to be considered out of order
		treshold: f64,
		/// Packets in a row requires for them to be considered an anomaly
		n_packets: usize,
		/// Seconds to cut off at the ends of the file
		#[arg(short, long)]
		cut: Option<f64>,
		/// Decimals to show for float values
		#[arg(short, long, default_value_t = 3)]
		decimals: usize,
	},
}

pub struct TrexData {
	pub transmit_times: Vec<f64>,
	pub arrival_times: Vec<f64>,
}

#[derive(Clone, ValueEnum)]
enum PlotMode {
	Latency,
	Jitter,
}

struct Anomaly {
	pub timestamp: f64,
	pub packets: u64,
	pub minimum_latency: f64,
	pub average_latency: f64,
	pub maximum_latency: f64,
}

fn main() {
	let cli: Cli = Cli::parse();
	match &cli.mode {
		Modes::Plot { .. } => mode_plot_data(cli),
		Modes::Validate { .. } => mode_validate(cli),
	}
}

fn mode_plot_data(cli: Cli) {
	if let Modes::Plot {
		plot_mode,
		output_file,
		cut,
	} = cli.mode
	{
		println!("Reading {}...", &cli.input_file);
		let mut data = get_file_timestamps(&cli.input_file)
			.expect(&format!("Failed to read file {}", &cli.input_file));
		println!("Read {} packet data points", data.transmit_times.len());
		utils::trexdata_to_latency(&mut data);
		// Used for cutting off warmup-time and cooldown-time
		let start_at;
		let end_at;
		if let Some(a) = cut {
			if a < 0.0 {
				panic!("Cut value must be positive");
			}
			start_at = (data.transmit_times.len() as f64 * a / data.transmit_times.last().unwrap())
				as usize;
			end_at = data.transmit_times.len() - start_at;
		} else {
			start_at = 0;
			end_at = data.transmit_times.len();
		}
		// Used for creating good values for the axises
		let mut highest_latency = utils::vector_max(&data.arrival_times[start_at..end_at]);
		let mut lowest_latency = utils::vector_min(&data.arrival_times[start_at..end_at]);
		// Used for printing stats
		let average_latency: f64 = utils::vector_avg(&data.arrival_times[start_at..end_at]);
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

		if let PlotMode::Jitter = plot_mode {
			// Map all values to inter-packet times
			let mut temp: f64;
			let mut prev: f64 = data.arrival_times[0];
			for i in 1..data.arrival_times.len() {
				temp = data.arrival_times[i].clone();
				data.arrival_times[i] -= prev;
				prev = temp;
			}
			data.arrival_times[0] = 0.0;
			highest_latency = utils::vector_max(&data.arrival_times[start_at..end_at]);
			lowest_latency = utils::vector_min(&data.arrival_times[start_at..end_at]);
		}

		let root = BitMapBackend::new(&output_file, (2400, 1600)).into_drawing_area();
		root.fill(&BLACK).unwrap();
		let mut chart = ChartBuilder::on(&root)
			.set_label_area_size(LabelAreaPosition::Left, 160)
			.set_label_area_size(LabelAreaPosition::Bottom, 100)
			.caption(
				match plot_mode {
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
			.y_desc(match plot_mode {
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
}

fn mode_validate(cli: Cli) {
	if let Modes::Validate {
		treshold,
		n_packets,
		cut,
		decimals,
	} = cli.mode
	{
		let mut data = TrexDataFile::new(&cli.input_file).expect("Failed to read file");
		// Used for cutting off warmup-time and cooldown-time
		let first_point = data.next().expect("Failed to read point data");
		let last_point = data.get_last_point().expect("Failed to read point data");
		data.reset().ok();
		let total_duration = last_point.0 - first_point.0;
		println!("Reading {} packet data points", data.len());
		let start_at = match cut {
			Some(c) => {
				if c < 0.0 {
					panic!("Cut value must be positive");
				}
				(data.len() as f64 * c / total_duration) as usize
			}
			None => 0,
		};
		let end_at = data.len() - start_at;
		// Can finally start looking for anomalies
		let mut anomaly_buffer: Vec<(f64, f64)> = vec![];
		let mut anomalies: Vec<Anomaly> = vec![];
		let mut processed = 0;
		let data_to_use = data
			.skip(start_at)
			.take(end_at - start_at)
			.map(|(transmit, arrival)| (transmit - first_point.0, arrival - first_point.0));
		for (transmit, arrival) in data_to_use {
			processed += 1;
			let latency = (arrival - transmit) * 1_000_000.0;
			if latency >= treshold {
				// Probably part of an anomaly, save it for when it's over
				anomaly_buffer.push((transmit, arrival));
				continue;
			}
			if anomaly_buffer.len() == 0 {
				// Normal packet, and not the end of an anomaly
				continue;
			}
			// If we get here, it's the end of an anomaly. Store it
			if anomaly_buffer.len() < n_packets {
				// Anomaly not long enough
				anomaly_buffer.clear();
				continue;
			}
			let anomaly_latencies = anomaly_buffer
				.iter()
				.map(|(transmit, arrival)| (arrival - transmit) * 1_000_000.0)
				.collect::<Vec<f64>>();
			anomalies.push(Anomaly {
				timestamp: anomaly_buffer[0].0,
				packets: anomaly_buffer.len() as u64,
				minimum_latency: utils::vector_min(&anomaly_latencies),
				average_latency: utils::vector_avg(&anomaly_latencies),
				maximum_latency: utils::vector_max(&anomaly_latencies),
			});
			anomaly_buffer.clear();
		}
		println!("Processed {} packets", processed);
		if anomalies.len() == 0 {
			println!("No anomalies found!");
		} else {
			for anomaly in anomalies {
				println!(
					"{:.5$}: {:>6} packets, {:.5$}/{:.5$}/{:.5$} µs min/avg/max",
					anomaly.timestamp,
					anomaly.packets,
					anomaly.minimum_latency,
					anomaly.average_latency,
					anomaly.maximum_latency,
					decimals,
				);
			}
		}
	}
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
