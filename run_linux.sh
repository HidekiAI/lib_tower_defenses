#!/bin/bash
# terms such as gnome-terminal does not work too well with ncurses (unsure why),
# so will launch xterm explicitly
echo "# DISPLAY=$DISPLAY"
_PWD=$(pwd)

xterm -e bash -c "cd ${_PWD} ; cargo run --bin lib_tower_defense"

