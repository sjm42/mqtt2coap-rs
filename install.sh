#!/bin/sh

set -x
set -e

PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
export PATH

tgt=$HOME/mqtt2coap/bin
rsync -var target/release/mqtt2coap $tgt/

exit 0
# EOF
