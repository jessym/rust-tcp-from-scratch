#!/usr/bin/env bash

###
### This script is ran INSIDE the VM to execute the Rust networking application
###

iface="mytun"

[[ -f "$HOME/.cargo/env" ]] && {
  source "$HOME/.cargo/env"
}
command -v "cargo" > /dev/null || {
  echo " > Installing rust"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
  sudo apt update -y
  sudo apt install -y build-essential
}

echo " > Building"
set -e
cargo build
mv ./target/debug/rust_networking ~
sudo setcap cap_net_admin=eip ~/rust_networking

echo " > Starting application in the background"
~/rust_networking &
pid="$!"

handle_exit_and_clean_up() {
  kill -SIGTERM "$pid" &> /dev/null
  rm -rf ~/rust_networking
}
trap handle_exit_and_clean_up EXIT
trap handle_exit_and_clean_up SIGINT

echo " > Configuring network"
while ! ip link show "$iface" &> /dev/null; do
  sleep 0.1
done
sudo ip addr add 192.168.0.1/24 dev "$iface"
sudo ip link set up dev "$iface"

echo " > Waiting for PID $pid"
wait "$pid"