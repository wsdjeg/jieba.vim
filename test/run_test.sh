#!/bin/bash
[ ! -d "../venv" ] && python3 -m venv ../venv
source ../venv/bin/activate
pip install -q -r requirements.txt
echo "=== vim integration tests ==="
VIM_BIN_NAME=vim pytest
echo
echo "=== nvim integration tests ==="
VIM_BIN_NAME=nvim pytest
