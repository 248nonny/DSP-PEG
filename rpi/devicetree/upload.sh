dtc -I dts -O dtb -o bcm2710-rpi-zero-2-w-dsp.dtb bcm2710-rpi-zero-2-w-dsp.dtc &&\
scp bcm2710-rpi-zero-2-w-dsp.dtb peg:~/ &&\

ssh "dsp@peg" "
set -x
sudo rm /boot/firmware/bcm2710-rpi-zero-2-w-dsp.dtb
sudo mv /home/dsp/bcm2710-rpi-zero-2-w-dsp.dtb /boot/firmware
sudo reboot
"
