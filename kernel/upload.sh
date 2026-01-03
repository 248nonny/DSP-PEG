#!/run/current-system/sw/bin/bash

REMOTE_USER=dsp
REMOTE_HOST=peg
REMOTE_PATH=/home/dsp/kernel-driver
MODULE_NAME=dsp_peg_kdrv
KDIR=/lib/modules/6.12.25+rpt-rpi-v8

# Remove previous build.
ssh "${REMOTE_USER}@${REMOTE_HOST}" "
set -x
rm -rf ${REMOTE_PATH}
mkdir -p ${REMOTE_PATH}/src
" &&\
\
echo -e "\n\ncopying source files to device:" &&\
\
scp ./src/* "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PATH}/src" && \
scp ./include/* "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PATH}/src" &&\
scp ./Makefile "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PATH}" &&\
\
echo -e "\n\ncompiling and inserting module:\n" &&\
ssh -t "${REMOTE_USER}@${REMOTE_HOST}" "
set -x
cd '${REMOTE_PATH}' &&\
make &&\
sleep 1 &&\
sudo mkdir -p ${KDIR}/kernel/drivers/extra &&\
sudo cp '${REMOTE_PATH}/build/'${MODULE_NAME}'.ko' ${KDIR}/kernel/drivers/extra &&\
sudo depmod -a &&\
echo "${MODULE_NAME}" | sudo tee /etc/modules
"
