scp ./dsp-peg-ui.service peg:/home/dsp
scp ./xinitrc peg:/home/dsp/.xinitrc

ssh "dsp@peg" "
set -x
sudo mv dsp-peg-ui.service /etc/systemd/system/
"
