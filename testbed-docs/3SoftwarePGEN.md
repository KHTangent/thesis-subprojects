# Packet Generator software setup

This section details how to set up the packet generator for performing tests. 

## Initial setup
These are the steps needed for creating the traffic generator for the first time.

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
6. Create a TRex configuration file somewhere, with the following contents. Replace the
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

TRex is now installed and ready to use. 

## Initializing TRex
In addition to the initial setup, the port configuration 
must be applied after every reboot by using the `dpdk_configure_ports.py` script:

```bash
cd trex-core/scripts
sudo ./dpdk_setup_ports.py --cfg path/to/config.yaml
```


TRex is now functional, and can be used as-is. Continue to the [Device Under Test setup](4SoftwareDUT.md) 
section to set up the device under test.

After making sure TRex works, it is recommended to spend some time on tuning the setup. This is described 
in the [Packet Generator validation and tuning](6TuningPGEN.md) section.


