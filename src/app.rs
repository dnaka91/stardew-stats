use gloo_file::{callbacks::FileReader, File, FileList};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::stardew;

pub struct App {
    upload_task: Option<FileReader>,
    content: Option<String>,
}

pub enum Msg {
    File(File),
    Loaded(String),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            upload_task: None,
            content: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::File(file) => {
                let callback = ctx.link().callback(Msg::Loaded);
                let task = gloo_file::callbacks::read_as_text(&file, move |res| {
                    callback.emit(res.unwrap())
                });
                self.upload_task = Some(task);
                true
            }
            Msg::Loaded(data) => {
                let save_game = stardew::load(&data);
                self.upload_task = None;
                self.content = Some(match save_game {
                    Ok(sg) => format!("{:#?}", sg),
                    Err(e) => format!("{:?}", e),
                });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_change = ctx.link().callback(|event: Event| {
            let target = event.target().expect("event should have target");
            let files = target
                .unchecked_into::<HtmlInputElement>()
                .files()
                .expect("target should have file list");
            let file = FileList::from(files)
                .first()
                .expect("file list shouldn't be empty")
                .clone();

            Msg::File(file)
        });

        html! {
            <section class="section">
                <div class="container">
                    <h1 class="title">{"Figure out what's in your save game file"}</h1>
                    <div class="block file">
                        <label class="file-label">
                            <input class="file-input" type="file" onchange={on_change} />
                            <span class="file-cta">
                                <span class="file-label">
                                    { "Choose a file to scan" }
                                </span>
                            </span>
                        </label>
                    </div>
                    <div class="block content">
                    <pre>
                        {
                            if let Some(content) = &self.content {
                                content.as_ref()
                            } else {
                                "Content will be displayed here"
                            }
                        }
                    </pre>
                    </div>
                </div>
            </section>
        }
    }
}
