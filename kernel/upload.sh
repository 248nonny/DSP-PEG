#!/run/current-system/sw/bin/bash

. ./options_upload.sh

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
sudo rmmod '${MODULE_NAME}'
sleep 1 &&\
sudo insmod '${REMOTE_PATH}/build/'${MODULE_NAME}'.ko' &&\
sudo dmesg | tail -20
"
