mkv-rename
==========

`mkv-rename` reads the creation date out of a video file and prepends it as a
UNIX epoch time stamp to the filename, so it will sort by name properly.

E.g. `IMG_4818.mkv` → `1673759524 IMG_4818.mkv`

**Note:** Despite the name the following formats are supported:

- Matroska (`mkv`)
- MPEG4 (`mov`, `m4v`, `mp4`)

This was a kind of a one-off script/tool built to deal with videos I'd pulled
off my iPhone and camera. I wanted them to all sort properly by creation date
no matter which device they came from.

Usage
-----

```
ARGS:
    <paths>...
      Files to process

OPTIONS:
    -n, --dry-run
      Don't rename files, just print what would be done

    -t, --tz-offset <offset>
      Offset in hours (can be fractional) to add to timestamps read from file

      Some cameras appear to store the creation date in local time, without a timezone.
      This flag allows those times to be adjusted.

    -h, --help
      Prints help information.
```
