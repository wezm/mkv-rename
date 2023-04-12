mkv-rename
==========

`mkv-rename` reads the creation date out of an Matroska (`mkv`) file and prepends
it as a UNIX epoch timestamp to the filename so it will sort by name properly.

E.g.

```
IMG_4818.mkv -> 1673759524 IMG_4818.mkv
```

This was a kind of a one-off script/tool built to deal with videos I'd pulled off
my iPhone and converted to `mkv`. The filenames that iOS gave the original videos
didn't follow creation order.

Usage
-----

```
ARGS:
    <paths>...
      Files to process

OPTIONS:
    -n, --dry-run
      Don't rename files, just print what would be done

    -h, --help
      Prints help information.
```
