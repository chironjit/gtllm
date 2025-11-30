use super::common::{ChatDisplay, ChatInput};
use crate::utils::{InputSettings, Message, Theme};
use dioxus::prelude::*;

#[component]
pub fn PvP(
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
                empty_title: "PvP Mode".to_string(),
                empty_description: "2 bots compete, 1 moderator judges - Watch AI models debate".to_string(),
            }

            ChatInput {
                theme,
                input_settings,
                on_send: move |text| on_send.call(text),
            }
        }
    }
}
