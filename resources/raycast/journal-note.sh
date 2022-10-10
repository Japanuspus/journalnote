#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Journal Note
# @raycast.mode silent

# Optional parameters:
# @raycast.icon 🤖
# @raycast.argument1 { "type": "text", "placeholder": "Note header", "optional": true }
# @raycast.argument2 { "type": "text", "placeholder": "Note text", "optional": true }

# Documentation:
# @raycast.description Journal Note
# @raycast.author Janus
# @raycast.authorURL https://insignificancegalore.net/

jn=/Users/janus/.cargo/bin/journalnote

if [ -z "$1" ]
then
  "$jn" "$2"
else
  "$jn" --header "$1" "$2"
fi