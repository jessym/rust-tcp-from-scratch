#!/usr/bin/env bash

###
### This script is used for opening an SSH session to a running Tart VM
###
### It should use your SSH public key, but VM credentials = admin / admin
###
vm="rust_networking_vm"
ssh_opts="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null"
ssh -qt $ssh_opts "admin@$(tart ip "$vm")" "cd /mnt; bash --login"
