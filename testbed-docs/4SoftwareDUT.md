# Device Under Test software setup

No matter what the device under test is, it must follow a few requirements: 

- Have two or more network interfaces
- Be able to route packets between them

We used a Linux desktop during our testing, but any device that meets the above requirements 
can be used. We will give a general procedure for configuration of the DUT, then show 
how we accomplished it on our Arch Linux desktop.

## General procedure

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

## Example: Linux desktop

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


Now that the DUT is ready, it is finally time to run some tests. Continue to the 
[Running the tests](5Running.md) section.

