#!/bin/sh
# Copyright (c) The Aptos Foundation
# SPDX-License-Identifier: Apache-2.0

apt-get update
apt-get install --no-install-recommends -y nano net-tools tcpdump iproute2 netcat ngrep atop gdb strace curl
