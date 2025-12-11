use dioxus::prelude::*;

#[component]
pub fn PhaseIndicator(phase_name: String, phase_number: u8, badge_color: String) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-2 mb-4",

            // Phase number badge
            div {
                class: "flex items-center justify-center w-8 h-8 rounded-full {badge_color} text-white font-bold text-sm",
                "{phase_number}"
            }

            // Phase name
            div {
                class: "text-lg font-semibold text-[var(--color-base-content)]",
                "{phase_name}"
            }
        }
    }
}
