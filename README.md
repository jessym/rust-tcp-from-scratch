# Rust TCP From Scratch

This is a learning project for implementing the TCP stack from scratch.

It uses the Linux TUN/TAP facility for implementing a network stack in userspace, rather than the kernel.

Tested with a MacOS host machine, using Tart VM for virtualizing a Linux environment.

## Getting Started

```bash
# 1. Spin up a Linux VM (the current working dir is mounted to /mnt)
./vm_start.sh

# 2. Optional; SSH into the Linux VM from a different terminal window as well
./vm_connect.sh

# 3. Make a real TCP request
nc 192.168.0.2 80
```

## Third Party Network Analysis Tool

This is useful as a comparison tool, to double-check whether our packet parsing works correctly.

To monitor traffic on a custom interface, use **tshark** (command line version of Wireshark).

```bash
tshark -i <interface>
```
