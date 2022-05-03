Imgur album downloader written in Rust.

```
imgur-downloader 0.1.0
download imgur albums and galleries

The album is downloaded into a directory named after the album id. Files are named after their
position in the album. Existing files are skipped if they have the correct size as reported by
imgur.

USAGE:
    imgur-downloader <ALBUM>

ARGS:
    <ALBUM>
            the album or gallery id or full url

            Examples:
            - vNOUshX
            - https://imgur.com/gallery/vNOUshX

OPTIONS:
    -h, --help
            Print help information

    -V, --version
            Print version information
```
