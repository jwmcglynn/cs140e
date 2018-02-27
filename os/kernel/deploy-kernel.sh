#!/bin/bash
BASEDIR=$(dirname "$0")

TTYPATH=/dev/tty.SLAB_USBtoUART
BAUD=115200

cd $BASEDIR
make install
RESULT=$?
if [ $RESULT -eq 0 ]; then
    screen $TTYPATH $BAUD
else
    echo "Build failed"
    exit $RESULT
fi
