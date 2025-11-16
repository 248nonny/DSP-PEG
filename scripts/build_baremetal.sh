#!/usr/bin/env bash

cargo build --release --package baremetal --target aarch64-unknown-none &&\
# mkdir -p build &&\
# aarch64-none-elf-objcopy -O binary ./baremetal/target/aarch64-unknown-none/release/bare_metal_pi_zero ./build/dsp_peg_fw.bin

## Copy firmware binary to rpi over ssh.
# ssh "dsp@peg" "
# set -x
# mkdir -p /home/dsp/firmware
# " && \
# scp build/dsp_peg_fw.bin peg:/home/dsp/firmware/ &&\

# ssh "dsp@peg" "
# set -x
# sudo rm /lib/firmware/dsp_peg_fw.bin
# sudo mv /home/dsp/firmware/dsp_peg_fw.bin /lib/firmware
# "
