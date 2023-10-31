#!/bin/bash

printf "\x1bc" && cargo run $1 ${2:-"input.txt"} ${3:-"output.qoi"}