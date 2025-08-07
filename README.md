# Raspberry Pi Keypulse

This is a small program that pulses on a Raspberry Pi GPIO pin when a key on
an attached keyboard is pressed. Pulses can be toggled active/inactive with
Ctrl+Shift+Super+S.

Input device, pin number, pulse length and whether pulses should be active or
inactive after program start can all be specified on the command line. See
`rpi-keypulse --help` for more information.

# Build Instructions

On a Raspberry Pi with rust installed, just do

    cargo build --release

If you want to tinker with it on a PC, you'll want to a debug build without
GPIO, in which case it'll just print "pin up/down" to stdout. In that case,
use

    cargo build --no-default-features
