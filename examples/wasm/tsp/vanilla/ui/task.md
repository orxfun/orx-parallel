# Create Readme

Create README.md.

Note that this the UI part of a demo for using orx-parallel and web workers with wasm. The readme must explain how to wire up `wasm_bindings`, how to create the search worker. Regular UI development details are not relevant.

The readme must be brief and highlight important points so that users can conveniently use orx-parallel in their own projects.

Start mentioning about used framework in this example.

Do not forget important details in json and config files.

Finally, mention that in this example, we create a thread pool per computation, because we create thread pool per search-worker module. Mention that it is possible to have a persistent search worker instead but often the overhead is negligible.

