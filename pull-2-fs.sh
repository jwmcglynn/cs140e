#!/bin/sh
cd "$(dirname "$0")"
git subtree pull --prefix 2-fs/ https://web.stanford.edu/class/cs140e/assignments/2-fs/skeleton.git master
