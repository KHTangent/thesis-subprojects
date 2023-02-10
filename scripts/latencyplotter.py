#!/usr/bin/env python3

# Simple script to plot the latency data files generated in my TRex fork. 
# Usage: python3 latencyplotter.py path/to/timestamps/file
# Opens a Matplotlib window
# Optionally, a second parameter may be given to save the figure as a png instead
# Set the second parameter to "s" to only print stats, or to "1" to only pick the 100th second

import matplotlib.pyplot as plt
from math import sqrt
import struct
from sys import argv


def main(): 
	if len(argv) < 2:
		print("Please spesify a filename")
		return
	filename = argv[1]
	transmit_times, latencies = read_doubles_file(filename)
	
	total_duration = round(transmit_times[-1] - transmit_times[0])
	one_second_length = len(transmit_times)//total_duration
	average_latency = sum(latencies)/len(latencies)
	variance = sum(map(lambda e: (e-average_latency)**2, latencies))/len(latencies)
	standard_deviation = sqrt(variance)
	print(f"""
  Summary
===========
Test duration (s):        {total_duration}
Number of packets:        {len(latencies)}
Average latency (s):      {average_latency}
Average latency (µs):     {average_latency * 10**6}
Standard deviation (s):   {standard_deviation}
Standard deviation (µs):  {standard_deviation * 10**6}
	""")

	if len(argv) > 2 and argv[2] == "s":
		# Print only stats
		return
	# The arrival time is relative to something, simply set the arrival time of the first packet as 0
	x = list(map(lambda e: e-transmit_times[0], transmit_times))
	# Values are in seconds, scale to microseconds
	y = list(map(lambda e: e * 10**6, latencies))
	# y = [(latencies[i]-latencies[i-1])*10**6 for i in range(1, len(transmit_times))]
	# y.append(0)
	
	plt.style.use("dark_background")
	if len(argv) > 2 and argv[2] == "1":
		first_index = (total_duration // 2 - 1) * one_second_length
		last_index = first_index + one_second_length
		plt.plot(x[first_index:last_index], y[first_index:last_index], "go", markersize=0.2)
	else:
		plt.plot(x, y, "go", markersize=0.2)
	plt.xlabel("Transmit time (s)")
	plt.ylabel("Latency (µs)")
	
	if argv[-1].endswith(".png"):
		plt.savefig(argv[-1])
	else:
		plt.show()

def read_doubles_file(filename: str):
	with open(filename, "rb") as file:
		transmit_times = []
		latencies = []
		while True:
			try:
				# The timestamp file contains raw doubles, where every other value is arrival time 
				# and measured latency
				raw_transmit = file.read(8)
				raw_arrival = file.read(8)
				transmit = struct.unpack("d", raw_transmit)[0]
				arrival = struct.unpack("d", raw_arrival)[0]
				transmit_times.append(transmit)
				latencies.append(arrival-transmit)
			except:
				break
	return transmit_times, latencies

if __name__ == "__main__":
	main()
