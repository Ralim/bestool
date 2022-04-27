# BES programming tool

Rough around the edges minimal python script to load code into the BES2300 over the uart.

This is built by capturing the traffic from the windows programming tool.
The `programmer.bin` is just from a uart capture of the payload the tool sends.
This file is obviously copyright BES.
The rest of this code & notes is released under MIT licence.

## Usage

Generally; usage is as simple as :

`./bestool.py program-watch /path/to/firmware.bin /dev/ttyUSB0`
This will wait for sync, program the device, then drop to a uart monitor.
