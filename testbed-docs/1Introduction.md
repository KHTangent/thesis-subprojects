# Creating a testbed for testing network devices suitability for real-time applications

These documents describe how to re-create the testbed I used in my master's thesis. The testbed 
can be used to see how well a device can handle real-time applications, by measuring "anomalies" 
in traffic handling. An anomaly is defined as "A group of N consecutive packets that have a 
latency over a threshold T".

The guide has several parts, split into multiple files. To have a full testbed setup, it is 
recommended to follow them in order. The parts are:

1. [Introduction](1Introduction.md): (this file)
2. [Hardware Setup](2Hardware.md): Hardware requirements, and how to set it up.
3. [Packet Generator software setup](3SoftwarePGEN.md): Software installation and configuration
   on the packet generator.
4. [Device Under Test setup](4SoftwareDUT.md): Software installation and configuration
   on the device under test. Contains the general procedure, and a specific example for a 
   Linux desktop.
5. [Test running and analysis](5Running.md): How to run the tests, and how to analyze the results.
6. [Packet Generator validation and tuning](6TuningPGEN.md): How to validate the packet generator
   setup, and how to tune it for optimal performance.

To get started, continue to the [Hardware Setup](2Hardware.md) section.



