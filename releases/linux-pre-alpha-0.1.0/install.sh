#!/bin/bash

cp -v ./sml /usr/bin
if [ $? -eq 0 ]; then
   echo SML Successfully Installed!
else
   echo This script must be run with permissions
fi
