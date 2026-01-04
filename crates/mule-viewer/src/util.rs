#[cfg(feature = "web")]
pub fn execute_async<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(feature = "native")]
pub fn execute_async<F: Future<Output = ()> + 'static>(f: F) {
    todo!("native code impl")
}
