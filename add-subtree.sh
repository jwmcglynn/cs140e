#!/bin/sh

if [ -z "$1" ]; then
    echo "No repository URL provided."
    echo "Usage: ./add-subtree.sh repository-url folder"
    exit 1
fi

if [ -z "$2" ]; then
    echo "No path provided."
    echo "Usage: ./add-subtree.sh repository-url folder"
    exit 1
fi

REPO=$1
SUBTREE_NAME=$2

BASEDIR=$(dirname $0)
cd $BASEDIR

git subtree add --prefix=$SUBTREE_NAME/ $REPO master