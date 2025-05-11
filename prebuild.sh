#!/bin/bash
set -ex
cd dep/libuv
./autogen.sh
./configure --prefix="`pwd`/../../_build" --with-pic --enable-static --disable-shared
make -j8
make install
