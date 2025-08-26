# Control a Honeywell USBHID scanner

## Usage

```
Usage: rust_hon_scanner.exe [OPTIONS]

Options:
-m, --mask <MASK>        Optional: The name (or part of it) of the device to look for
-v, --vid <VID>          Optional: USB Vendor identifier. This takes precedence over the option --mask
-p, --pid <PID>          Optional: USB Product identifier. This takes precedence over the option --mask
-l, --list               Optional: Show a list of available devices and exit
-s, --scan               Optional: Scan a barcode
-i, --info               Optional: Send REVINFO
-c, --command <COMMAND>  Optional: The command to send to the selected scanner
-h, --help               Print help
-V, --version            Print version
```

## Notes:
* If a mask is given, then the vid and pid are not neccessary
* If vid is used, then the pid must be given as well.

Example of the --list output:

```
Connected USB devices:
0: 1602g                Honeywell Imaging & Mobility             (0c2e:0db3)
1: HIDI2C Device        Microsoft                                (044e:120b)
2: M720_Triathlon       Logitech                                 (046d:b01
```

## Developer notes

Q: project rust_hon_scanner fails to compileren on Raspberry Pi with "The system library 'libudev' required bij crate 'hdiapi' was not found
A: sudo apt install libudev-dev



