#!/bin/sh
cd "$(dirname "$0")"
git subtree pull --prefix 1-shell/ https://web.stanford.edu/class/cs140e/assignments/1-shell/skeleton.git master
