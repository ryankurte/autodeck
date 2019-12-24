#!/bin/bash

VAL=$(( ( RANDOM % 2 ) ))

if [ "$VAL" == "0" ]; then
    exit 0
else
    exit 1
fi
