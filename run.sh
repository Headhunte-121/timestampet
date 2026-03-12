#!/bin/bash
cd "$(dirname "$0")"
nohup python3 run.py > /dev/null 2>&1 &
