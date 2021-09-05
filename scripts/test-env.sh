#!/bin/bash

ROOT=$(realpath $(dirname $0)/..)

BASE_PATH=$ROOT/test

if [ -d $BASE_PATH ]; then
        rm -rf $BASE_PATH;
fi

mkdir -p $BASE_PATH/{origin,local}

for r in repo1 repo2 repo3; do
        echo "Creating $r";
        d=$BASE_PATH/origin/$r.git
        mkdir -p $d
        cd $d
        git init -b master
        echo "1st $r README" > $d/README.md
        git add README.md
        git commit -m "Initial commit @ $r"
        sleep 0.1

        echo "2nd commit" >> $d/README.md
        git add README.md
        git commit -m "Commit 2 @ $r"
        sleep 0.1

        echo "3rd commit" >> $d/README.md
        git add README.md
        git commit -m "Commit 3 @ $r"
        sleep 0.1
done
