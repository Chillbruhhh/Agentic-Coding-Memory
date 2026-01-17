#!/bin/bash
cd server
cargo build 2>&1 | tail -20
