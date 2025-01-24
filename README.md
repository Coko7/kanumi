# 🎨 kanumi-rs

Select / filter images from your terminal.

## Usage

```console
coko7@example:~$ kanumi -h
Select / filter image collections

Usage: kanumi [OPTIONS] [DIRECTORY]

Arguments:
  [DIRECTORY]  Root directory to use to search for collection of images

Options:
  -t, --type <NODE_TYPE>               Restrict output to specified node type [default: image] [possible values: directory, image]
  -m, --metadata-file <METADATA_PATH>  Path to the CSV file containing individual image scores
  -s, --score <SCORE_RANGE>            Only show images with score contained within this range
  -W, --width <WIDTH_RANGE>            Only show images with a width contained within this range
  -H, --height <HEIGHT_RANGE>          Only show images with a height contained within this range
  -c, --conf-gen                       Generate default configuration
  -v, --verbose...                     Increase logging verbosity
  -q, --quiet...                       Decrease logging verbosity
  -h, --help                           Print help
```

### Examples

1. Select images with width >= 1920, height >= 1080, with a score < 2
```console
coko7@example:~$ kanumi -t img -W 1920.. -H 1080.. -s 0..1
coko7@example:~$ kanumi -t img -W 1920.. -H 1080.. -s ..1
```

2. Select tiny images with a score of exactly 5:
```console
coko7@example:~$ kanumi -t i -W ..50 -H ..50 -s 5
coko7@example:~$ kanumi -t i -W ..50 -H 0..50 -s 5..5
coko7@example:~$ kanumi -t i -W 0..50 -H ..50 -s 5..5
```
