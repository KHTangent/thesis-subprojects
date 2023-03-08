#!/usr/bin/env python3

# Script for use with my thesis to perform many tests with TRex. Could probably have been implemented as a 
# bash script instead, but Python is easy.
# This script must be ran as Root
# Make sure to modify the constants here at the top before trying to run the script

# Absolute path to the scripts subfolder of trex-core
TREX_SCRIPTS_PATH = "/home/rtuser/Development/trex-core/scripts/"
# Absolute path to where you want the generated data files to be stored
EXPORT_PATH = "/home/rtuser/media/disk/trex-tests/stock-router"
# How you want to start TRex, with taskset and configuration file
TREX_BASE_COMMAND = "taskset 0x3 ./_t-rex-64 --cfg /home/rtuser/Development/trexscripts/testconfig.yml"

# Name of your current test
TEST_NAME = "stock-router"
# How long each test should be, in seconds
TEST_DURATION = 600
# Values for the -l parameter, which represent latency packets sent per second
PACKETS_PER_SECOND = [150000, 300000, 600000]
# How many tests to run for each value of PACKETS_PER_SECOND
TESTS_PER_CONFIGURATION = 5


import os
from shlex import quote
from time import sleep

def main(): 
	os.chdir(TREX_SCRIPTS_PATH)
	os.system(f"mkdir -p {EXPORT_PATH}")
	for pps in PACKETS_PER_SECOND:
		for i in range(TESTS_PER_CONFIGURATION):
			params = f" --lo -l {pps} -f cap2/dns.yaml -d {TEST_DURATION}"
			os.system(TREX_BASE_COMMAND + params)
			sleep(5) # Make sure TRex has fully closed
			timetamps_filename = next(filter(lambda e: e.startswith("timestamps"), os.listdir()))
			new_filename = f"{TEST_NAME}-d{TEST_DURATION}-l{pps}-{i}.data"
			new_filename = os.path.join(EXPORT_PATH, quote(new_filename))
			os.system(f"mv {timetamps_filename} {new_filename}")
			sleep(5)

if __name__ == "__main__":
	main()
