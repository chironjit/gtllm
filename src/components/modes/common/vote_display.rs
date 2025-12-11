use dioxus::prelude::*;

#[component]
pub fn VoteDisplay(
    voter_id: String,
    voted_for: Option<String>,
    raw_response: Option<String>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 p-3 rounded-lg bg-[var(--color-base-100)] border border-[var(--color-base-300)]",

            // Voter ID
            div {
                class: "font-mono text-sm text-[var(--color-base-content)]/70 min-w-[200px]",
                "{voter_id}"
            }

            // Vote arrow
            div {
                class: "text-[var(--color-base-content)]/50",
                "→"
            }

            // Voted for or error
            if let Some(vote) = voted_for {
                div {
                    class: "flex items-center gap-2 flex-1",
                    div {
                        class: "font-mono text-sm font-semibold text-[var(--color-primary)]",
                        "{vote}"
                    }
                    // Checkmark
                    div {
                        class: "text-green-500 text-xl",
                        "✓"
                    }
                }
            } else {
                div {
                    class: "flex items-center gap-2 flex-1",
                    div {
                        class: "text-sm text-red-600",
                        "Invalid vote"
                    }
                    // X mark
                    div {
                        class: "text-red-500 text-xl",
                        "✗"
                    }
                }
            }

            // Optional raw response (for debugging/transparency)
            if let Some(raw) = raw_response {
                if !raw.is_empty() {
                    details {
                        class: "ml-auto",
                        summary {
                            class: "text-xs text-[var(--color-base-content)]/50 cursor-pointer hover:text-[var(--color-base-content)]/70",
                            "raw"
                        }
                        div {
                            class: "mt-2 p-2 bg-[var(--color-base-200)] rounded text-xs font-mono max-w-xs overflow-x-auto",
                            "{raw}"
                        }
                    }
                }
            }
        }
    }
}
