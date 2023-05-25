use clap::{Parser, Subcommand, ValueEnum};
use plotters::prelude::*;
use rand::{self, Rng, SeedableRng};
use std::vec;

use crate::{file_handler::TrexDataFile, utils::Tally};

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
	/// Generate plots of individual packets
	Plot {
		/// What to plot
		#[arg(short, long)]
		plot_mode: PlotMode,
		/// PNG file to use for output
		#[arg(short, long)]
		output_file: String,
		/// Seconds to cut off at the ends of the file
		#[arg(short, long)]
		cut: Option<f64>,
	},
	/// Test for latency anomalies, and optionally generate a plot of them
	Validate {
		/// Treshold (in µs) for a packet to be considered out of order.
		/// Ignored if -d is given
		#[arg(short, long, default_value_t = 500.0)]
		treshold: f64,
		/// Consecutive packets required for them to be considered an anomaly
		#[arg(short, long, default_value_t = 2)]
		n_packets: usize,
		/// Maximum deviation from average latency to be considered an anomaly. 
		/// For example, a value of 3 means that a packet with latency 3 times the average 
		/// latency is considered part of an anomaly.
		/// Slower than -t, since the file must be read twice to obtain the average.
		#[arg(short, long)]
		deviation: Option<f64>,
		/// Seconds to cut off at the ends of the file, to avoid warmup and cooldown deviations
		#[arg(short, long)]
		cut: Option<f64>,
		/// Decimals to print for float values
		#[arg(long, default_value_t = 3)]
		decimals: usize,
		/// Generate an anomaly plot to this PNG file.
		/// If omitted, plot generation is skipped.
		#[arg(short, long)]
		output_file: Option<String>,
		/// By default, the program will list all anomalies. Set this option to only print a 
		/// summary of the anomalies.
		#[arg(long, default_value_t = false)]
		summary_only: bool,
	},
}

#[derive(Clone, ValueEnum)]
enum PlotMode {
	/// Plot latency in µs
	Latency,
	/// Plot time since last packet in µs
	Jitter,
}

struct Anomaly {
	pub timestamp: f64,
	pub tally: Tally,
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
		let mut data = TrexDataFile::new(&cli.input_file).expect("Failed to read file");
		// Used for cutting off warmup-time and cooldown-time
		let first_point = data.next().expect("Failed to read point data");
		let last_point = data.get_last_point().expect("Failed to read point data");
		data.reset().ok();
		let total_duration = last_point.0 - first_point.0;
		println!(
			"Reading {} packet data points, test lasted {:.1} seconds",
			data.len(),
			total_duration
		);
		// Used for cutting off warmup-time and cooldown-time
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
		let mut arrival_times = Vec::with_capacity(data.len());
		let mut latencies = Vec::with_capacity(data.len());
		let mut tally = Tally::new();
		for (transmit, arrival) in data {
			arrival_times.push(transmit - first_point.0);
			let latency_us = (arrival - transmit) * 1_000_000.0;
			latencies.push(latency_us);
			tally.add(latency_us);
		}
		// Used for printing stats
		let average_latency = tally.avg();
		println!(
			"Packets range from {} to {} µs, with an average of {} µs",
			&tally.min, &tally.max, &average_latency
		);
		println!("Standard deviation is {} µs", &tally.stddev());

		if let PlotMode::Jitter = plot_mode {
			// Map all values to inter-packet times
			let mut temp: f64;
			let mut prev: f64 = latencies[0];
			tally = Tally::new();
			for i in 1..latencies.len() {
				temp = latencies[i].clone();
				latencies[i] -= prev;
				prev = temp;
				tally.add(latencies[i]);
			}
			latencies[0] = 0.0;
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
				arrival_times[start_at].round() - 1.0..arrival_times[end_at - 1].round() + 1.0,
				// Expand the range of the y axis a bit
				(0.9 * tally.min)..(1.1 * tally.max),
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
				(start_at..end_at).map(|i| Pixel::new((arrival_times[i], latencies[i]), &GREEN)),
			)
			.unwrap();
	}
}

fn mode_validate(cli: Cli) {
	if let Modes::Validate {
		treshold,
		n_packets,
		deviation,
		cut,
		decimals,
		output_file,
		summary_only,
	} = cli.mode
	{
		let mut data = TrexDataFile::new(&cli.input_file).expect("Failed to read file");
		// Used for cutting off warmup-time and cooldown-time
		let first_point = data.next().expect("Failed to read point data");
		let last_point = data.get_last_point().expect("Failed to read point data");
		data.reset().ok();
		let total_duration = last_point.0 - first_point.0;
		if !summary_only {
			println!("Reading {} packet data points", data.len());
		}
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
		let anomaly_treshold = match deviation {
			Some(d) => {
				let data = TrexDataFile::new(&cli.input_file).expect("Failed to read file");
				let mut tally = Tally::new();
				data.skip(start_at)
					.take(end_at - start_at)
					.for_each(|(t, a)| tally.add((a - t) * 1_000_000.0));
				tally.avg() * (1.0 + d / 100.0)
			}
			None => treshold,
		};
		// Can finally start looking for anomalies
		let mut anomaly_buffer: Vec<(f64, f64)> = vec![];
		let mut anomalies: Vec<Anomaly> = vec![];
		let mut total_tally = Tally::new();
		let mut anomaly_summary_packets = Tally::new();
		let mut anomaly_summary_max = Tally::new();
		let mut anomaly_summary_avg = Tally::new();
		let data_to_use = data
			.skip(start_at)
			.take(end_at - start_at)
			.map(|(transmit, arrival)| (transmit - first_point.0, arrival - first_point.0));
		for (transmit, arrival) in data_to_use {
			let latency = (arrival - transmit) * 1_000_000.0;
			total_tally.add(latency);
			if latency >= anomaly_treshold {
				// Probably part of an anomaly, save it for when it's over
				anomaly_buffer.push((transmit, arrival));
				continue;
			}
			if anomaly_buffer.len() == 0 {
				// Normal packet, and not the end of an anomaly
				continue;
			}
			// If we get here, it's the end of an anomaly. Store it if it's long enough
			if anomaly_buffer.len() < n_packets {
				// Anomaly not long enough
				anomaly_buffer.clear();
				continue;
			}
			let mut anomaly_tally = Tally::new();
			anomaly_buffer
				.iter()
				.map(|(transmit, arrival)| (arrival - transmit) * 1_000_000.0)
				.for_each(|v| anomaly_tally.add(v));
			anomaly_summary_packets.add(anomaly_tally.count as f64);
			anomaly_summary_avg.add(anomaly_tally.avg());
			anomaly_summary_max.add(anomaly_tally.max);
			anomalies.push(Anomaly {
				timestamp: anomaly_buffer[0].0,
				tally: anomaly_tally,
			});
			anomaly_buffer.clear();
		}
		let average_latency = total_tally.avg();
		if !summary_only {
			if anomalies.len() == 0 {
				println!("No anomalies found!");
			} else {
				for anomaly in anomalies.iter() {
					println!(
						"{:.5$}: {:>6} packets, {:.5$}/{:.5$}/{:.5$} µs min/avg/max",
						anomaly.timestamp,
						anomaly.tally.count,
						anomaly.tally.min,
						anomaly.tally.avg(),
						anomaly.tally.max,
						decimals,
					);
				}
			}
		}
		println!("===== Summary =====");
		println!(
			"Total duration: {:.1$} s",
			total_duration - 2.0 * cut.unwrap_or(0.0),
			decimals
		);
		println!("Total packets: {:.1$}", total_tally.count, decimals);
		println!(
			"Latency (min/avg/max): {:.3$}/{:.3$}/{:.3$} µs",
			total_tally.min, average_latency, total_tally.max, decimals
		);
		println!(
			"Standard deviation: {:.1$} µs",
			total_tally.stddev(),
			decimals
		);
		println!(
			"Anomaly treshold: {:.2$} µs, n >= {}",
			anomaly_treshold, n_packets, decimals
		);
		println!("Total anomalies: {}", anomalies.len());
		if anomalies.len() > 0 {
			println!(
				"Average anomaly duration: {:.1$} packets",
				anomaly_summary_packets.avg(),
				decimals
			);
			println!(
				"Anomaly average latency (min/avg/max): {:.3$}/{:.3$}/{:.3$} µs",
				anomaly_summary_avg.min,
				anomaly_summary_avg.avg(),
				anomaly_summary_avg.max,
				decimals
			);
			println!(
				"Anomaly maximum latency (min/avg/max): {:.3$}/{:.3$}/{:.3$} µs",
				anomaly_summary_max.min,
				anomaly_summary_max.avg(),
				anomaly_summary_max.max,
				decimals
			);
		}

		if let Some(output_file) = output_file {
			let root = BitMapBackend::new(&output_file, (2400, 1600)).into_drawing_area();
			root.fill(&BLACK).unwrap();
			let mut chart = ChartBuilder::on(&root)
				.set_label_area_size(LabelAreaPosition::Left, 160)
				.set_label_area_size(LabelAreaPosition::Bottom, 100)
				.caption("Anomaly plot", ("sans-serif", 50).into_font().color(&WHITE))
				.margin(30)
				.build_cartesian_2d(
					// Calculate time bounds for x-axis as whole values
					cut.unwrap_or(0.0).floor()..(total_duration - cut.unwrap_or(0.0)).ceil(),
					// Expand the range of the y axis a bit
					(0.9 * total_tally.min)..(1.1 * total_tally.max),
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
				.draw_series([
					plotters::element::PathElement::new(
						[
							(cut.unwrap_or(0.0), average_latency),
							(total_duration - cut.unwrap_or(0.0), average_latency),
						],
						&GREEN,
					),
					plotters::element::PathElement::new(
						[
							(cut.unwrap_or(0.0), anomaly_treshold),
							(total_duration - cut.unwrap_or(0.0), anomaly_treshold),
						],
						&GREEN,
					),
				])
				.unwrap();
			chart
				.draw_series([
					plotters::element::Text::new(
						"Average latency",
						(cut.unwrap_or(0.0), average_latency),
						("sans-serif", 24).into_font().color(&WHITE),
					),
					plotters::element::Text::new(
						"Anomaly treshold",
						(cut.unwrap_or(0.0), anomaly_treshold),
						("sans-serif", 24).into_font().color(&WHITE),
					),
				])
				.unwrap();
			// How wide the lines marking endpoints and averages on anomalies should be
			// Spesified as a part of the X axis, so make it dependent on the total duration
			let marker_lines_width =
				((total_duration - cut.unwrap_or(0.0)).ceil() - cut.unwrap_or(0.0).floor()) * 0.002;
			chart
				.draw_series(anomalies.iter().map(|anomaly| {
					let color = get_random_color(anomaly.timestamp);
					let avg = anomaly.tally.avg();
					plotters::element::PathElement::new(
						[
							(anomaly.timestamp - marker_lines_width, anomaly.tally.min),
							(anomaly.timestamp + marker_lines_width, anomaly.tally.min),
							(anomaly.timestamp, anomaly.tally.min),
							(anomaly.timestamp, avg),
							(anomaly.timestamp - marker_lines_width, avg),
							(anomaly.timestamp + marker_lines_width, avg),
							(anomaly.timestamp, avg),
							(anomaly.timestamp, anomaly.tally.max),
							(anomaly.timestamp - marker_lines_width, anomaly.tally.max),
							(anomaly.timestamp + marker_lines_width, anomaly.tally.max),
						],
						&color,
					)
				}))
				.unwrap();
		} else if !summary_only {
			println!("No output file specified, will not generate plot");
		}
	}
}

fn get_random_color(seed: f64) -> RGBColor {
	let mut rng = rand::rngs::SmallRng::seed_from_u64(seed.to_bits());
	RGBColor(
		rng.gen_range(0..255),
		rng.gen_range(0..255),
		rng.gen_range(0..255),
	)
}
