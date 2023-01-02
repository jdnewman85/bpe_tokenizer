#!/bin/bash

set -euo pipefail
#set -x

# shellcheck disable=SC2046
$EDITOR $(fd --extension rs)
