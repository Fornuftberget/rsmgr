#!/bin/bash

ROOT=$(realpath $(dirname $0)/..)

BASE_PATH=$ROOT/test

for r in repo1 repo2 repo3; do
        d=$BASE_PATH/origin/$r.git
        cd $d

	git rebase HEAD~3
done
