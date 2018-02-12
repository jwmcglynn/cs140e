#!/bin/bash
BASEDIR=$(dirname "$0")

cp $BASEDIR/build/kernel.bin /Volumes/NO\ NAME/kernel8.img
diskutil unmountDisk /dev/disk2
