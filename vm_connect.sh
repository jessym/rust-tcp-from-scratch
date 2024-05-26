#!/usr/bin/env bash

###
### Tart VM: https://tart.run/
###
### This script is used for starting a VM, and opening an SSH session to it
### (necessary for executing the `run.sh` script)
###
### VM credentials = admin / admin
###
vm="rust_networking_vm"
ssh_opts="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null"
ssh -qt $ssh_opts "admin@$(tart ip "$vm")" "cd /mnt; bash --login"
