# The Bub's Brothers website (GitHub Pages)

This branch (`gh-pages`) is the project website: plain HTML, CSS and
JavaScript, no build step.

- `index.html`: what's new, screenshots.
- `download.html`: the download helper and the table of release files.
- `js/releases.js`: **the release's version and file list**. For a new
  release, change `VERSION` and the file names and sizes there (the
  download helper's file names are built in `js/download.js`, `PKG` and
  `recommend()`).
- `images/`: screenshots.

To publish: in the repository settings on GitHub, *Pages*, choose
"Deploy from a branch", branch `gh-pages`, folder `/ (root)`.
