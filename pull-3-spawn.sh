#!/bin/sh
cd "$(dirname "$0")"
git subtree pull --prefix 3-spawn/ https://web.stanford.edu/class/cs140e/assignments/3-spawn/skeleton.git master
