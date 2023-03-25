#!/usr/bin/env python3

# Script for use with my thesis to perform many tests with TRex. Could probably have been implemented as a
# bash script instead, but Python is easy.
# This script must be ran as Root
# Make sure to modify the constants here at the top before trying to run the script

# Name of your current test
from time import sleep
from shlex import quote
import os
TEST_NAME = "stock-router"
# Absolute path to where you want the generated data files to be stored
EXPORT_PATH = f"/home/rtuser/media/disk/trex-tests/{TEST_NAME}"
# Absolute path to the scripts subfolder of trex-core
TREX_SCRIPTS_PATH = "/home/rtuser/Development/trex-core/scripts/"
# How you want to start TRex, with taskset and configuration file
TREX_BASE_COMMAND = "taskset -c 0-3 ./_t-rex-64 --cfg /home/rtuser/Development/trexscripts/testconfig.yml"

# How long each test should be, in seconds
TEST_DURATION = 600
# Values for the -l parameter, which represent latency packets sent per second
# One latency packet is 62 bytes. A value of 190 kpps seems to give about 100 Mbps
PACKETS_PER_SECOND = [
	1900,  # 1 Mbps, probably won't trigger the NAPI
	19000,  # 10 Mbps, might ignore this one
	190000,  # 100 Mbps, good baseline
	1700000,  # 900 Mbps, the maximum we seem to be able to get
]
# How many tests to run for each value of PACKETS_PER_SECOND
RUNS_PER_CONFIGURATION = 15

# How long to wait for TRex to finish storing data before starting the next test
WAIT_DURATION = 30


def main():
	os.chdir(TREX_SCRIPTS_PATH)
	total_test_count = len(PACKETS_PER_SECOND) * RUNS_PER_CONFIGURATION
	total_duration = (
		TEST_DURATION * total_test_count +
		2 * WAIT_DURATION * total_test_count
	)
	print(f"Estimated total duration: {round(total_duration / 3600, 1)} hours")
	sleep(30)  # Allow the user to actually read the estimate before starting
	os.system(f"mkdir -p {EXPORT_PATH}")
	for pps in PACKETS_PER_SECOND:
		for run in range(RUNS_PER_CONFIGURATION):
			params = f" --lo -l {pps} -f cap2/dns.yaml -d {TEST_DURATION}"
			os.system(TREX_BASE_COMMAND + params)
			sleep(WAIT_DURATION)  # Make sure TRex has fully closed
			timetamps_filename = list(
				filter(lambda e: e.startswith("timestamps"), os.listdir()))[-1]
			new_filename = f"{TEST_NAME}-d{TEST_DURATION}-l{pps}-{run}.data"
			new_filename = os.path.join(EXPORT_PATH, quote(new_filename))
			os.system(f"mv {timetamps_filename} {new_filename}")
			sleep(WAIT_DURATION)


if __name__ == "__main__":
	main()
