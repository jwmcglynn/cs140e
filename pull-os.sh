#!/bin/sh
cd "$(dirname "$0")"
git subtree pull --prefix os/ https://web.stanford.edu/class/cs140e/os.git master
