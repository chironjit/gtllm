use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct VoteTallyProps {
    pub tallies: Vec<(String, usize, Vec<String>)>,  // (model_id, vote_count, voters)
    pub winners: Vec<String>,
}

#[component]
pub fn VoteTally(props: VoteTallyProps) -> Element {
    rsx! {
        div {
            class: "p-6 rounded-lg bg-[var(--color-base-100)] border-2 border-[var(--color-primary)]",

            // Header
            div {
                class: "text-xl font-bold text-[var(--color-base-content)] mb-4",
                if props.winners.len() > 1 {
                    "Winners (Tie)"
                } else if props.winners.len() == 1 {
                    "Winner"
                } else {
                    "Results"
                }
            }

            // Winners section
            if !props.winners.is_empty() {
                div {
                    class: "mb-6 p-4 bg-green-500/10 border border-green-500 rounded-lg",
                    for winner_id in props.winners.iter() {
                        div {
                            class: "flex items-center gap-2 text-lg font-semibold text-green-600 mb-1 last:mb-0",
                            div { class: "text-2xl", "🏆" }
                            div { class: "font-mono", "{winner_id}" }
                        }
                    }
                }
            }

            // Vote tally section
            div {
                class: "space-y-3",
                div {
                    class: "text-sm font-semibold text-[var(--color-base-content)]/70 mb-2",
                    "Vote Breakdown:"
                }

                for (model_id, vote_count, voters) in props.tallies.iter() {
                    div {
                        class: "p-3 rounded-lg bg-[var(--color-base-200)] border border-[var(--color-base-300)]",

                        div {
                            class: "flex items-center justify-between mb-2",
                            div {
                                class: "font-mono text-sm font-semibold text-[var(--color-base-content)]",
                                "{model_id}"
                            }
                            div {
                                class: "flex items-center gap-2",
                                div {
                                    class: "text-sm font-bold text-[var(--color-primary)]",
                                    {format!("{vote_count} vote{}", if *vote_count == 1 { "" } else { "s" })}
                                }
                                if props.winners.contains(model_id) {
                                    div { class: "text-xl", "🏆" }
                                }
                            }
                        }

                        if !voters.is_empty() {
                            div {
                                class: "text-xs text-[var(--color-base-content)]/60",
                                "Voted by: {voters.join(\", \")}"
                            }
                        }
                    }
                }
            }
        }
    }
}
