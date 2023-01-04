#!/bin/bash

# Set the URL and the JSON payload
url="http://localhost:8000/tokenize_new"
json="{\"input\": \"$1\"}"

# Send the POST request
curl -X POST -H "Content-Type: application/json" -d "$json" "$url"
