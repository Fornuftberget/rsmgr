#!/bin/bash

ROOT=$(realpath $(dirname $0)/..)

BASE_PATH=$ROOT/test

for r in repo1 repo2 repo3; do
        d=$BASE_PATH/origin/$r.git
        cd $d

        echo "4:th commit" >> $d/README.md
        git add README.md
        git commit -m "Commit 4 @ $r"

        echo "5:th commit" >> $d/README.md
        git add README.md
        git commit -m "Commit 5 @ $r"

        echo "6th: commit" >> $d/README.md
        git add README.md
        git commit -m "Commit 6 @ $r"

        echo "7:th commit" >> $d/README.md
        git add README.md
        git commit -m "Commit 7 @ $r"
done
