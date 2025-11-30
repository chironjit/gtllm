use super::common::{ChatDisplay, ChatInput};
use crate::utils::{InputSettings, Message, Theme};
use dioxus::prelude::*;

#[component]
pub fn Choice(
    theme: Signal<Theme>,
    messages: Signal<Vec<Message>>,
    input_settings: Signal<InputSettings>,
    on_send: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col h-full",

            ChatDisplay {
                theme,
                messages,
                empty_title: "LLM's Choice Mode".to_string(),
                empty_description: "LLMs decide to collaborate or compete - AI autonomy at its finest".to_string(),
            }

            ChatInput {
                theme,
                input_settings,
                on_send: move |text| on_send.call(text),
            }
        }
    }
}
