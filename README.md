# spreadsheet

A spreadsheet written in Rust + WASM + React.

Backend is written in Rust with the Actyx web framework. It's responsible for persisting spreadsheet cell data and for doing websocket shenanigans (i.e. collaboration features).

Data is persisted in Postgresql for convenience -- the Diesel ORM seemed nice.

Frontend is written in React for the UI, and Rust w/ WASM for the actual spreadsheet engine.

All in all, we have a fat client, and a slim but stateful backend.

# setup

Install rust + WASM + node toolchains as described in the [official docs](https://rustwasm.github.io/book/game-of-life/setup.html).

# running

For backend

`cd backend`

`cargo run`

For frontend

`cd frontend`

To compile the rust sources into WASM

`wasm-pack build`

To serve the website

`cd www`  
`npm install`  
`npm start`

You can then go to `localhost:3000`
