Install prerequisites:

  rustup toolchain add nightly
  rustup target add wasm32-unknown-unknown --toolchain=nightly

Possibly?
  #cargo +nightly install cargo-web

Compile and serve with::

  make
