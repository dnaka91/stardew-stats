use std::str;

use yew::{
    prelude::*,
    services::{
        reader::{File, FileData, ReaderTask},
        ReaderService,
    },
};

use crate::stardew;

pub struct App {
    link: ComponentLink<Self>,
    upload_task: Option<ReaderTask>,
    content: Option<String>,
}

pub enum Msg {
    File(File),
    Loaded(FileData),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            upload_task: None,
            content: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::File(file) => {
                let callback = self.link.callback(Msg::Loaded);
                let task = ReaderService::new().read_file(file, callback).unwrap();
                self.upload_task = Some(task);
                true
            }
            Msg::Loaded(data) => {
                let content = str::from_utf8(&data.content).unwrap();
                let save_game = stardew::load(content);
                self.upload_task = None;
                self.content = Some(match save_game {
                    Ok(sg) => format!("{:#?}", sg),
                    Err(e) => format!("{:?}", e),
                });
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <section class="section">
                <div class="container">
                    <h1 class="title">{"Figure out what's in your save game file"}</h1>
                    <div class="block file">
                        <label class="file-label">
                            <input class="file-input" type="file" onchange=self.link.callback(move |value| {
                                if let ChangeData::Files(files) = value {
                                    let file = js_sys::try_iter(&files)
                                        .unwrap()
                                        .unwrap()
                                        .map(|v|File::from(v.unwrap()))
                                        .next()
                                        .unwrap();
                                    Msg::File(file)
                                } else {
                                    panic!("no files!")
                                }
                            })
                            />
                            <span class="file-cta">
                                <span class="file-label">
                                    { "Choose a file to upload" }
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
