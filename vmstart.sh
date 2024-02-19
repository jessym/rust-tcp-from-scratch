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

##
## Helper functions
##
function is_vm_running() {
  if [[ "$(tart list --source local --format json | jq -r ".[] | select(.Name == \"$vm\").State")" == "running" ]]; then
    return 0
  fi
  return 1
}

##
## Create VM
##
#tart delete "$vm"    # <-- Uncomment this line below to recreate the VM
tart list --quiet --source local | grep "$vm" --quiet || {
  tart clone ghcr.io/cirruslabs/ubuntu:latest "$vm"
}

##
## Find my public key
##
public_key="$(find ~/.ssh/ -type f -name '*.pub' -maxdepth 1)"
if [[ ! "$(echo "$public_key" | grep -c '^')" -eq "1" ]]; then
  echo "Aborting, encountered zero or more than one public key ❌"
  exit 1
fi

##
## Start VM
##
is_vm_running && tart stop "$vm"
tart run --dir="$(pwd)" "$vm" &
pid="$!"
while ! is_vm_running; do
  echo "Waiting for VM $pid to start..."
  sleep 1
done
sleep 1

##
## SSH to server
##
while ! ssh -q $ssh_opts "admin@$(tart ip "$vm")" 'cat > ~/.ssh/authorized_keys' < "$public_key"; do
  echo "Waiting for VM $pid to establish SSH connection..."
  sleep 1
done
ssh -qt $ssh_opts "admin@$(tart ip "$vm")" \
  "sudo mount -t virtiofs com.apple.virtio-fs.automount /mnt; cd /mnt; bash --login"

##
## Shut down VM
##
kill -SIGTERM "$pid"
wait "$pid" &> /dev/null
