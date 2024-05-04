#!/usr/bin/env bash

###
### This script is ran INSIDE the VM to execute the Rust networking application
###

iface="mytun" # This should correspond with main.rs
binary="rust-tcp-from-scratch" # This should correspond with Cargo.toml

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
mv "./target/debug/$binary" ~
sudo setcap cap_net_admin=eip "$HOME/$binary"

echo " > Starting application in the background"
"$HOME/$binary" &
pid="$!"

handle_exit_and_clean_up() {
  kill -SIGTERM "$pid" &> /dev/null
  rm -rf "${HOME:?}/$binary"
}
trap handle_exit_and_clean_up EXIT
trap handle_exit_and_clean_up SIGINT

echo " > Configuring network"
while ! ip link show "$iface" &> /dev/null; do
  sleep 0.1
done
sudo ip addr add 192.168.0.1/24 dev "$iface"
sudo ip link set dev "$iface" up

echo " > Waiting for PID $pid"
wait "$pid"