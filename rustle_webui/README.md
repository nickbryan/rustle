# rustle_webui

This crate provides a web-based user interface for the Rustle editor.

It uses `xterm.js` to render the editor in a web browser. The application is compiled to WebAssembly and interacts with
the browser's DOM.

## Building and Running

This project uses `trunk` to build and serve the application.

To install `trunk`:

```sh
cargo install trunk
```

To serve the application:

```sh
trunk serve --open
```

This will build the application, start a local server, and open the application in your browser. Any changes to the code
will be automatically rebuilt and reloaded.
