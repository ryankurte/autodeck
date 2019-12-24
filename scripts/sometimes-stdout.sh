#!/bin/bash

VAL=$(( ( RANDOM % 2 ) ))

if [ "$VAL" == "0" ]; then
    echo "ok"
else
    echo "error"
fi
