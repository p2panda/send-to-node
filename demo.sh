#!/bin/bash

cat 001-schema-field-definition-name.json | cargo run -- -k ./key-demo.txt
cat 002-schema-definition-venue.json | cargo run -- -k ./key-demo.txt
cat 003-create-venue.json | cargo run -- -k ./key-demo.txt
cat 004-update-venue.json | cargo run -- -k ./key-demo.txt
cat 005-delete-venue.json | cargo run -- -k ./key-demo.txt
cat 006-create-venue.json | cargo run -- -k ./key-demo.txt
cat 007-update-conflict-1.json | cargo run -- -k ./key-demo.txt
cat 008-update-conflict-2.json | cargo run -- -k ./key-demo.txt
cat 009-update-conflict-merge.json | cargo run -- -k ./key-demo.txt
cat 010-schema-field-definition-venue.json | cargo run -- -k ./key-demo.txt
cat 011-schema-definition-event.json | cargo run -- -k ./key-demo.txt
cat 012-create-event.json | cargo run -- -k ./key-demo.txt
