# Hardware setup

The testbed uses two computers, one to generate traffic (the Packet GENerator, PGEN), and one 
device to be tested (DUT). Both devices have some minimum requirements to be able to work 
under this test setup. For the PGEN, the requirements are:

- A network interface card with at least two interfaces, that are supported by Cisco TRex. 
  See table 5 on 
  [this page of the TRex manual](https://trex-tgn.cisco.com/trex/doc/trex_manual.html#_hardware_recommendations)
  for a list of supported network cards.
- An installation of a Linux distribution. A fresh installation of Arch Linux was used during testing, but other 
  distributions should work too.
- In addition, it is recommended to have at least 32 GB of RAM, and a "powerful" desktop CPU. 
  An Intel Xeon W-1270P running at 5.10 GHz was used during testing.

For the device under test, the requirements are:

- Two network interfaces
- The ability to route or forward packets between them

During testing, two point-to-point connections should be made between the PGEN and the DUT, 
so traffic can flow both ways through different interfaces.

Once you have the hardware setup ready, continue to the 
[Packet Generator software setup](3SoftwarePGEN.md) section.

