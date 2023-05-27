# Tuning the packet generator

This section gives some tips on how to tune the packet generator (PGEN) to get more accurate 
results.

## Run TRex in loopback mode to reduce outside interference
While tuning the PGEN, it is recommended to run TRex in loopback mode. This means that the 
PGEN will send packets to itself, instead of sending them to the DUT. This has the advantage that 
the PGEN is not affected by the DUT's performance, and that the PGEN can be tuned without 
needing to have the DUT connected. Running in loopback mode requires a different configuration 
file than the one used for normal testing.

First, connect the two physical network interfaces on the PGEN to each other.

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

Run the `dpdk_setup_ports.py` script as described earlier, but use this new `loopback.yaml` config file 
as input parameter. If an error occurs because the ports are already bound, reboot the PGEN, and try again.

```bash
cd trex-core/scripts
sudo ./dpdk_setup_ports.py --cfg path/to/loopback.yaml
```


## Suggested optimizations

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



