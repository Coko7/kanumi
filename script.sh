#!/usr/bin/env bash

./target/release/pixer-rs "$HOME/Pictures/Wallpapers" \
    -m "$HOME/Pictures/Wallpapers/nsfw_scores.csv" \
    -t image -s 3..5 | fzf --preview "$SCRIPTS/fzf-preview.sh {}"
