# Cisco TRex as a testbed for suitability of running real-time applications

This document describes how to re-create the testbed I used in my master's thesis. The testbed 
can be used to see how well a device can handle real-time applications, by measuring "anomalies" 
in traffic handling. An anomaly is defined as "A group of N consecutive packets that have a 
latency over a treshold T".

The full test setup has several parts: 

- A desktop computer with two free network interfaces, to generate traffic
- The device to be tested (DUT)
- A fork of the Cisco TRex application with support for exporting all latency packets. 
  [KHTangent/trex-core](https://github.com/KHTangent/trex-core)
- An application for plotting and analyzing the generated data files from TRex. Located in 
  [the data-postprocessr subfolder of this repository](https://github.com/KHTangent/thesis-subprojects/tree/master/data-postprocessor)

Cisco TRex is used to generate test traffic, and measure how well it is handled by the Device 
Under Test (DUT), while the data-postprocessor is used to interpret the results. 


## Hardware setup

The testbed uses two computers, one to generate traffic (the Packet GENerator, PGEN), and one 
device to be tested (DUT). Both devices have some minimum requirements to be able to work 
under this test setup. For the PGEN, the requirements are:

- A network interface card with at least two interfaces, that are supported by Cisco TRex. 
  See table 5 on 
  [this page of the TRex manual](https://trex-tgn.cisco.com/trex/doc/trex_manual.html#_hardware_recommendations)
  for a list of supported network cards.
- An installation of a Linux distribution. A fresh install of Arch Linux was used during testing, but other 
  distributions should work too.
- In addition, it is recommended to have at least 32 GB of RAM, and a "powerful" desktop CPU. 
  An Intel Xeon W-1270P running at 5.1 GHz was used during testing.

For the device under test, the requirements are:

- Two network interfaces
- The ability to route or forward packets between them

During testing, two point-to-point connections should be made between the PGEN and the DUT, 
so traffic can flow both ways through different interfaces.

### PGEN setup

This subsection details how to set up the packet generator for performing measurements. 

Initial setup, only needs to be performed once:

1. Install a Linux distro on the PGEN. Arch Linux is used in this example.
2. Install `git` and GCC (if you receive errors later, install `gcc8` and try again)
3. Clone the trex-core fork: `git clone https://github.com/KHTangent/trex-core`
4. Build TRex:
   ```bash
   cd trex-core/linux_dpdk
   # Either
   ./b configure
   ./b build
   # Or, in case the above commands give errors
   CXX=g++-8 CC=gcc-8 ./b configure
   ./b build
   ```
5. Find the ID's of the network cards you want to use.
   1. cd into the scripts directory of TRex: `cd ../scripts`
   2. Run `sudo ./dpdk_setup_ports.py -s` to see a list of available ports.
      ```
      Network devices using kernel driver
      ===================================
      0000:00:1f.6 'Ethernet Connection (11) I219-LM' if=eno1 ...
      0000:01:00.0 'Ethernet Controller X710 for 10GBASE-T' if=enp1s0f0 ...
      0000:01:00.1 'Ethernet Controller X710 for 10GBASE-T' if=enp1s0f1 ...
      ```
      In our case, we want to use the Intel X710 interfaces, which here have ID's 
	  `01:00.0` and `01:00.1`
   3. Create a TRex configuration file somewhere, with the following contents. Replace the
      port ID's with the ones you found in the previous step.
	  ```yaml
      - port_limit: 2
        version: 2
        interfaces: ["01:00.0", "01:00.1"] # Replace if needed
        port_info:
          - ip         : 11.11.11.2
            default_gw : 11.11.11.1
          - ip         : 12.12.12.2
            default_gw : 12.12.12.1
       ```

TRex is now installed and ready to use. In addition to the initial setup, the port configuration 
must be applied after every reboot by using the `dpdk_configure_ports.py` script.

```bash
cd trex-core/scripts
sudo ./dpdk_configure_ports.py --cfg path/to/config.yaml
```

### DUT Setup

TODO

### Data-postprocessor setup

This can be performed on any machine, for example on the PGEN, or a separate device.

1. Install Rust. The simplest way is usually to use [`rustup`](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Clone this repo, and build the data-processor
   ```bash
   git clone https://github.com/KHTangent/thesis-subprojects
   cd thesis-subprojects/data-postprocessor
   cargo build --release
   ```

The executable should now be available as 
`thesis-subprojects/data-postprocessor/target/release/data-postprocessor`.
It can be freely moved to a more convenient location if you want, it does not have any 
external dependencies.


## Running a test

After configuring TRex and the DUT, you are now ready to run tests. A general test 
has two stages: 

1. Using TRex to generate traffic, measure forwarding latency of each packet, and export
   to a data blob.
2. Analyzing this blob using the data-postprocessor.

To run a test:

```bash
cd trex-core/scripts
sudo ./_t-rex-64 --cfg path/to/config.yaml --lo -l 2000000 -f cap2/dns.yaml -d 60
```

Explanation of parameters:

- `--lo` Send only latency traffic. Latency traffic is the only traffic we can obtain 
  full latency stats for, so only send this
- `-l 2000000` Send 2 million latency packets every second. May need tweaking for your setup
- `-f cap2/dns.yaml` TRex requires an input file to run the mode we use, but since we use 
  `--lo`, the contents doesn't affect anything. `cap2/dns.yaml` is a simple minimal file.
- `-d 60` Run test for 60 seconds

After the test has finished, two files will be placed in your `trex-core/scripts` directory, 
titled `timestamps-[date]-p0` and `timestamps-[date]-p1`. These contain raw values for transmit 
and arrival times of all latency packets generated by TRex. The two files contains mostly the 
same data, so one of them can safely be deleted. These files are accepted by the 
data-postprocessor. 


## Viewing results

TODO

## Tuning and optimizing the PGEN

Since the PGEN is a full Linux system, and not just a hardware generator, the OS can produce 
spikes by itself, altering the results. This needs to be accounted for. But we can make some 
effort to minimize the spikes. This section has a few tips.

### Run TRex on an isolated CPU core. 

1. Add the following to your GRUB configuration to isolate two CPU cores from the kernel:
   ```
   isolcpus=0,1
   ```
2. Reboot
3. Make sure your CPU cores are isolated by checking the contents of `/sys/devices/system/cpu/isolated`
4. Prefix all TRex commands with `taskset 0x3`. For example, to run the test mentioned above: 
   ```
   taskset 0x3 sudo ./_t-rex-64 --cfg path/to/config.yaml --lo -l 2000000 -f cap2/dns.yaml -d 60
   ```

### Run TRex in loopback mode to see what to expect
This can help give an estimation of how many spikes are generated by TRex. 
Create a copy of your TRex config file titled `loopback.yaml`, and set your IP addresses like this: 
```yaml
- port_limit: 2
  version: 2
  interfaces: ["01:00.0", "01:00.1"] # Replace if needed
  port_info:
  - ip         : 11.11.11.2
    default_gw : 12.12.12.2
  - ip         : 12.12.12.2
    default_gw : 11.11.11.2
```
