#!/bin/bash

cat 001-schema-field-definition-name.toml | cargo run -- -k ./key-demo.txt
cat 002-schema-definition-venue.toml | cargo run -- -k ./key-demo.txt
cat 003-create-venue.toml | cargo run -- -k ./key-demo.txt
cat 004-update-venue.toml | cargo run -- -k ./key-demo.txt -d 00205446251e9d235fa7214cbd8a9a67e8d971bebecd875199cfe43fc5f038b8c6f5
cat 005-delete-venue.toml | cargo run -- -k ./key-demo.txt -d 00205446251e9d235fa7214cbd8a9a67e8d971bebecd875199cfe43fc5f038b8c6f5
cat 006-create-venue.toml | cargo run -- -k ./key-demo.txt
cat 007-update-conflict-1.toml | cargo run -- -k ./key-demo.txt -d 0020785779ab68f23aebad5b5cc35dea32f8cbea57a028ccdb3d96345e21d2473de1
cat 008-update-conflict-2.toml | cargo run -- -k ./key-demo.txt -d 0020785779ab68f23aebad5b5cc35dea32f8cbea57a028ccdb3d96345e21d2473de1
cat 009-update-conflict-merge.toml | cargo run -- -k ./key-demo.txt -d 0020785779ab68f23aebad5b5cc35dea32f8cbea57a028ccdb3d96345e21d2473de1
cat 010-schema-field-definition-venue.toml | cargo run -- -k ./key-demo.txt
cat 011-schema-definition-event.toml | cargo run -- -k ./key-demo.txt
cat 012-create-event.toml | cargo run -- -k ./key-demo.txt
