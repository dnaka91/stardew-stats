#![recursion_limit = "512"]

mod app;
mod stardew;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<app::App>();
}
