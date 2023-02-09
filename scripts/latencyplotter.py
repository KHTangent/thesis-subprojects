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

if len(argv) < 2:
	print("Please spesify a filename")
	exit()

filename = argv[1]

with open(filename, "rb") as file:
	x = []
	y = []
	while True:
		try:
			# The timestamp file contains raw doubles, where every other value is arrival time 
			# and measured latency
			raw_arrival = file.read(8)
			raw_latency = file.read(8)
			arrival = struct.unpack("d", raw_arrival)[0]
			latency = struct.unpack("d", raw_latency)[0]
			x.append(arrival)
			y.append(latency)
		except:
			break

total_duration = round(x[-1] - x[0])
one_second_length = len(x)//total_duration
average_latency = sum(y)/len(y)
variance = sum(map(lambda e: (e-average_latency)**2, y))/len(y)
standard_deviation = sqrt(variance)

print(f"""
  Summary
===========
      Test duration (s):  {total_duration}
      Number of packets:  {len(y)}
    Average latency (s):  {average_latency}
   Average latency (µs):  {average_latency * 10**6}
 Standard deviation (s):  {standard_deviation}
Standard deviation (µs):  {standard_deviation * 10**6}
""")

if len(argv) > 2 and argv[2] == "s":
	exit()

# The arrival time is relative to something, simply set the arrival time of the first packet as 0
x_adjusted = list(map(lambda e: e-x[0], x))
# Values are in seconds, scale to milliseconds
y_scaled = list(map(lambda e: e * 10**6, y))

plt.style.use("dark_background")
if len(argv) > 2 and argv[2] == "1":
	first_index = (total_duration // 2 - 1) * one_second_length
	last_index = first_index + one_second_length
	plt.plot(x_adjusted[first_index:last_index], y_scaled[first_index:last_index], "go", markersize=0.2)
else:
	plt.plot(x_adjusted, y_scaled, "go", markersize=0.2)
plt.xlabel("Transmit time (s)")
plt.ylabel("Latency (µs)")

if argv[-1].endswith(".png"):
	plt.savefig(argv[-1])
else:
	plt.show()
