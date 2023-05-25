# Cisco TRex as a testbed for suitability of running real-time applications

This document describes how to re-create the testbed I used in my master's thesis. The testbed 
can be used to see how well a device can handle real-time applications, by measuring "anomalies" 
in traffic handling. An anomaly is defined as "A group of N consecutive packets that have a 
latency over a threshold T".

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
must be applied after every reboot by using the `dpdk_configure_ports.py` script:

```bash
cd trex-core/scripts
sudo ./dpdk_configure_ports.py --cfg path/to/config.yaml
```

### DUT Setup

No matter what the device under test is, it must follow a few requirements: 

- Have two or more network interfaces
- Be able to route packets between them

We used a Linux desktop during our testing, but any device that meets the above requirements 
can be used. We will give a general procedure for configuration of the DUT, then show 
how we accomplished it on our Arch Linux desktop.

1. Enable forwarding of packets on the device
2. Enable the two interfaces
3. Add static routes for the two interfaces used by TRex. In the previous section, we 
   configured TRex to use `11.11.11.2` as its IP address on one interface, and expects the DUT 
   to use `11.11.11.1` on the same interface. The same applies to the other interface, 
   where the PGEN uses `12.12.12.2` and expects the DUT to use `12.12.12.1`. Therefore, assign 
   a static IP address of `11.11.11.1/24` to the interface that will be used for receiving data, 
   and `12.12.12.1/24` for the other interface.
4. TRex will send packets with a source IP address in the `48.0.0.0/8` subnet, with a destination 
   address in `16.0.0.0/8`. Therefore, add static routes for these subnets, with the PGEN as the 
   gateway. 
5. Sometimes, static ARP routes must be added on the DUT to locate the PGEN. Find the MAC addresses 
   of the interfaces on the PGEN, and add static ARP routes for them on the DUT. 

If the DUT is a Linux device, it can be helpful to put all the required setup steps in a shell 
script, since many of them will have to be re-executed after every reboot. The script could look like 
this:

```bash
#!/bin/bash
# Enable forwarding of packets
sysctl -w net.ipv4.ip_forward=1

# Bring up our interfaces. Replace with actual interface names
ip link set enp1s0f0 up
ip link set enp1s0f1 up

# Add static IP addresses to the interfaces. Replace the interface names if needed
ip a add 11.11.11.1/24 dev enp1s0f0
ip a add 12.12.12.1/24 dev enp1s0f1

# Add static routes for the traffic from TRex. These commands should work as-is, if 
# you use the config given above
ip route add 48.0.0.0/8 via 12.12.12.2
ip route add 16.0.0.0/8 via 11.11.11.2

# Add static ARP entries. Replace the MAC addresses with the ones of the PGEN interfaces
arp -s 11.11.11.2 68:05:ca:df:09:26
arp -s 12.12.12.2 68:05:ca:df:09:27
```

Run this script as root after every reboot, and the DUT should be ready to receive 
traffic from TRex.


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
3. If you want to install the data-postprocessor system wide, run the following command:
   ```bash
   cargo install --bins --path .
   ```

If you chose to not install the executable system wide, it will be located in
`thesis-subprojects/data-postprocessor/target/release/data-postprocessor`.
It can be freely moved to a more convenient location if you want, it does not have any 
external dependencies.


## Running a test

After configuring TRex and the DUT, you are now ready to run tests. A general test 
has three stages: 

1. Configure the DUT to forward packets between the two interfaces, and apply any other configurations
   you want to try on the DUT.
2. Use TRex to generate latency measurement traffic, which is stored as a data blob.
3. Analyzing this blob using the data-postprocessor.

To run a test:

```bash
cd trex-core/scripts
sudo ./_t-rex-64 --cfg path/to/config.yaml --lo -l 190000 -f cap2/dns.yaml -d 60
```

Explanation of parameters:

- `--lo` Send only latency traffic. Latency traffic is the only traffic we can obtain 
  full latency stats for, so only send this
- `-l 190000` Send 190 thousand latency packets every second, giving about 100 Mbps of traffic
- `-f cap2/dns.yaml` TRex requires an input file to run the mode we use, but since we use 
  `--lo`, the contents doesn't affect anything. `cap2/dns.yaml` is a simple minimal file.
- `-d 60` Run test for 60 seconds

After the test has finished, a data blob will be placed in your `trex-core/scripts` directory, 
titled `timestamps-[date]-p0`. This file contains raw values for transmit 
and arrival times of all latency packets generated by TRex.
This file is accepted by the data-postprocessor.


### Run TRex in loopback mode to see what to expect
To make sure that TRex is working as expected, it is recommended to try running it in loopback mode.
To run in loopback mode, connect the two interfaces of the PGEN to each other. In addition, a separate 
config file for TRex is needed in loopback cases.

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

Run the `dpdk_setup_ports.py` script as described above, but use this new `loopback.yaml` config file 
as input parameter. Afterwards, running a test is only a matter of selecting the right config file.


## Viewing results

The data blobs can be analyzed using the data-postprocessor. The data-postprocessor has a 
help page that can be viewed by running `data-postprocessor --help`.

Examples of commands that can be run:

```bash
# Print a summary of anomalies in the data blob, and save a plot to plot.png. 
# Consider 2 consecutive packets with a latency of 500 µs an anomaly
# Cut away the first and last second of the data
data-postprocessor timestamps-[date]-p0 validate -n 2 -t 500 --summary-only -c 1 -o plot.png

# Print all of the anomalies in the data blob, and save a plot to plot.png. 
# Consider 2 consecutive packets with a latency of three times the average latency an anomaly
# Cut away the first and last five seconds of the data
data-postprocessor timestamps-[date]-p0 validate -n 2 -d 3 --summary-only -c 5 -o plot.png

# Plot latencies of all packets in the data blob. Include all data
data-postprocessor timestamps-[date]-p0 plot -p latency -o plot.png
```


## Optimizing the PGEN to reduce TRex-induced spikes

Even while running in loopback, it is possible that latency spikes appear in the test results. This 
can happen for various reasons, for example because of other processes running on the PGEN, or 
how TRex is implemented. To make results as accurate as possible, it is recommended to spend some time 
experimenting in loopback mode until you get a good baseline with minimal spikes. This section will 
give some suggestions.

### Minimize the number of processes running on the PGEN

A good starting point is to disable all services that are not needed for the PGEN to function.

The commands in this section assume that you are using `systemd` to manage your services. If you 
are not, you will need to adapt them to your system.

First, use a tool like `htop` to get an overview of what's running on the system. 
Many services can be disabled by simply running `systemctl disable <service>`.

In addition, it is recommended to disable X11, and any other graphical services, on the PGEN. 
TRex is a command-line application, so a desktop environment is not needed for it to function. 
To disable X11, run the following command:
```bash
sudo systemctl set-default multi-user.target
```
Reboot, and you should be taken to a terminal instead of a graphical login screen.

To undo this change at a later point, use the following command:
```bash
sudo systemctl set-default graphical.target
```

### Run TRex on an isolated CPU core. 

1. Add the following to your GRUB configuration to isolate four CPU cores from the kernel:
   ```
   isolcpus=0,1,2,3
   ```
2. Reboot
3. Make sure your CPU cores are isolated by checking the contents of `/sys/devices/system/cpu/isolated`
4. Prefix all TRex commands with `taskset -c 0-3`. For example, to run the test mentioned earlier: 
   ```
   taskset -c 0-3 sudo ./_t-rex-64 --cfg path/to/config.yaml --lo -l 190000 -f cap2/dns.yaml -d 60
   ```


### Switch to the preempt-rt kernel

The preempt-rt kernel is a real-time kernel, which is designed to minimize jitter and spikes by 
changing how the kernel handles scheduling, among other things.
In our experience, it has been very effective at reducing spikes from the PGEN.
Switching to the preempt-rt kernel is a bit distribution-dependent, so the following steps are 
only directly applicable to Arch Linux. 

1. Install the `linux-rt` package: `sudo pacman -S linux-rt`  
2. Regenerate your GRUB configuration: `sudo grub-mkconfig -o /boot/grub/grub.cfg`
3. Reboot

Once the system has booted, you can verify that you are running the preempt-rt kernel by running 
`cat /proc/version`. It should contain `rt` in the output.

