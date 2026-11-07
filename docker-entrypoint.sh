#!/bin/sh
# Refresh the shipped game data in the data volume, then start the server.
#
# /var/lib/dikumud is a volume holding both the shipped data (world, help,
# messages, ...) and what the game writes (plrs/, board.messages, bugs,
# ideas, typos, reboot). The shipped files are copied over on every start so
# a new image brings its world updates with it; files the game writes are
# never in the image, so they are left alone.
set -eu

DATA=/var/lib/dikumud
cp -R /usr/share/dikumud/lib/. "$DATA/"
mkdir -p "$DATA/plrs"

exec /usr/local/bin/dmserver -d "$DATA" "$@"
