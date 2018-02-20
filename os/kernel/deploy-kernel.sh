#!/bin/bash
BASEDIR=$(dirname "$0")

TTYPATH=/dev/tty.SLAB_USBtoUART
BAUD=115200

cd $BASEDIR
make install
screen $TTYPATH $BAUD
