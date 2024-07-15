#!/usr/bin/env bash

sudo singularity build --tmpdir "/home/nju" --force crawler.sif crawler.def \
&& cp crawler.sif full.sif \
&& singularity overlay create --size 30000 full.sif
