#!/bin/bash
BASEDIR=$(dirname "$0")

KERNEL=$BASEDIR/build/kernel.bin

TTYWRITE=$BASEDIR/../../1-shell/ttywrite/target/debug/ttywrite
TTYPATH=/dev/tty.SLAB_USBtoUART
BAUD=115200

cat $KERNEL | $TTYWRITE $TTYPATH --baud $BAUD
screen $TTYPATH $BAUD
