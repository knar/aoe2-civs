mod app;
mod components;
mod data;
#[cfg(test)]
mod data_tests;
mod model;
mod overview;
mod storage;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}