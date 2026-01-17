# loveletters

`loveletters` is a proof-of-concept static site generator (SSG).
`loveletters` is built in top of [`typst`](https://typst.app/), setting it apart from most existing SSGs.
You can read more on `loveletters` in [this blogpost](https://yacob.90degs2infty.dev/posts/hello-loveletters/).

For my love for letters.

## Status quo

`loveletters` is a proof-of-concept.
As such, expect it to render your content, panic or wipe your harddrive.
I'm exaggerating - but you get the point.
Functionality is limited as is code quality.

## Getting started

To try out `loveletters`, you need

- [`Rust`](https://rust-lang.org/) (see [Install Rust](https://rust-lang.org/tools/install/) for details on getting `Rust`) as well as
- some content to render.
  Content has to be arranged as shown in [this demo project](https://github.com/90degs2infty/loveletters-demo).
  Please also take into account below [note on the `@loveletters/loveletters` package](#a-note-on-the-lovelettersloveletters-package).

With the above prerequisites in place, issue:

```console
> git clone https://github.com/90degs2infty/loveletters
> cd loveletters
> cargo run -- <project-directory> <output-directory>
```

This will render your content and bundle it for static serving.
Obviously, you can also build `loveletters` using `cargo` and execute the built binary instead of `cargo run`ning it.

`loveletters` does _not_ include a webserver, so refer to your favorite webserver for local deployment of `loveletters`'s output.
If you don't have a favorite webserver, you can use e.g. [Static Web Server](https://static-web-server.net/).
In this case

```console
> static-web-server -p <port> -d <output-directory>
```

does the job.

With the webserver running, direct your browser to `http://localhost:<port>/`.

### A note on the `@loveletters/loveletters` package

Because of how `loveletters` currently locates `typst` packages from the `loveletters` namespace (namely, it does not provide `@loveletters/loveletters` automatically), you have to copy-paste the package's sources to your project directory by hand.
To do so, simply copy the content of this repository's subfolder `loveletters_util` to your project directory, placing it under `<project-dir>/packages/loveletters/0.1.0`.
You can then import said package using

```typ
#import "@loveletters/template:0.1.0": toplevel_template
```

from within your `typst` content.

Note that the package lookup is intended to be changed in future versions of `loveletters`, so that this copy-pasting becomes obsolete.


## Prior art

### `typst` glue-code

Glue-code to drive `typst` is heavily inspired by [`typst-as-library` by `@tfachmann`](https://github.com/tfachmann/typst-as-library) released under Apache-2.0 license.
Find their license [here](https://github.com/tfachmann/typst-as-library/blob/main/LICENSE).

## License

`loveletters` is licensed under MIT OR Apache License 2.0.
See [`LICENSE-MIT`](./LICENSE-MIT) or [`LICENSE-APACHE`](./LICENSE-APACHE) for details.
