#!/bin/bash
# terms such as gnome-terminal does not work too well with ncurses (unsure why),
# so will launch xterm explicitly
echo "# DISPLAY=$DISPLAY"
_PWD=$(pwd)


if ! [ -e target/debug/assets ] ; then
    mkdir target/debug/assets
fi
cp -R "assets/*" target/debug/assets
cp -R "lib/*" target/debug/

xterm -e bash -c "cd ${_PWD} ; cargo run --bin sdl2_view"

