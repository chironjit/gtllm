use super::common::{ChatDisplay, ChatInput};
use crate::utils::{InputSettings, Message, Theme};
use dioxus::prelude::*;

#[component]
pub fn Competitive(
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
                empty_title: "Competitive Mode".to_string(),
                empty_description: "All bots vote for the best (can't vote for their own) - Let democracy decide".to_string(),
            }

            ChatInput {
                theme,
                input_settings,
                on_send: move |text| on_send.call(text),
            }
        }
    }
}
