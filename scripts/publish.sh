#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 [-h|--help]"
  echo "Publishes crates to crates.io."
}

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 1
fi

# These are imported by other packages
ALL_PACKAGES="controllers cw-utils cw2"
SLEEP_TIME=30

for pack in $ALL_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    cargo publish
    # wait for these to be processed on crates.io
    echo "Waiting for publishing all packages"
    sleep $SLEEP_TIME
  )
done

echo "Everything is published!"
