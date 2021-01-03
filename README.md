A command line application to download an imgur albums. Written in Rust.

Takes one command line argument. This can be a full imgur album url or just the album id. Imgur galleries also work.

Creates a new folder with the id of the album and downloads all files of the album into the folder. Files are named by their position in the album with leading zeroes.

If a file already exists and the size matches what imgur reports then the file is not downloaded again. This is useful when a previous run was interrupted.

Why? Imgur does have a "download zip" button but for many albums it errors with "download link expired".
