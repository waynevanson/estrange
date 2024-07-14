#!/bin/sh
#
# collapse intermediary files between to relative locations
#
# `collapse <directory>`
#

# Removes "./<path>" and "/<path>" and return "<path>"
clean_directory_path() {
    local variable="$1"
    local variable="${variable#./}"
    local variable="${variable#/}"

    echo "$variable"
}

get_highest_ancestor() {
    echo "${1%%/*}"
}

for directory in "$@"
do
    # # move the directory contents to $PWD
    # # todo: name collisions on multiple
    mv -t "$PWD/" $directory/*

    root="$(clean_directory_path "$directory")"

    # root="${}"
    root="$(get_highest_ancestor "$root")"

    # # delete the stuff in between
    rm -rf "$root/"
done