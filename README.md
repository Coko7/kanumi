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
