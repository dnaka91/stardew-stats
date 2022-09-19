#![recursion_limit = "512"]

mod app;
mod stardew;

pub fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<app::App>();
}
