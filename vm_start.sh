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
  echo "⏳ Waiting for VM $pid to start..."
  sleep 1
done
echo "✅ VM $pid started!"
sleep 1

##
## Determine the IP address
##
vm_ip_addr=""
while [[ -z "$vm_ip_addr" ]]; do
  echo "⏳ Waiting for IP address..."
  vm_ip_addr="$(tart ip "$vm" 2>/dev/null)"
  sleep 1
done
echo "✅ IP address $vm_ip_addr determined!"
sleep 1

##
## Save personal SSH public key to the VM server
##
while ! ssh -q $ssh_opts "admin@$vm_ip_addr" 'cat > ~/.ssh/authorized_keys' < "$public_key"; do
  echo "⏳ Waiting for VM $pid to accept SSH public key..."
  sleep 1
done
echo "✅ VM $pid has accepted SSH public key!"
sleep 1

##
## SSH into the VM
##
ssh -qt $ssh_opts "admin@$vm_ip_addr" \
  "sudo mount -t virtiofs com.apple.virtio-fs.automount /mnt; cd /mnt; bash --login"

##
## Shut down VM
##
kill -SIGTERM "$pid"
wait "$pid" &> /dev/null
