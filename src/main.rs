use leptos::prelude::*;

mod data;
mod math;
mod components;

fn main() {
    mount_to_body(components::app::App);
}
