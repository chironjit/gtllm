use crate::utils::{Theme, OpenRouterClient, CreditsData};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn Header(
    theme: Signal<Theme>,
    on_toggle_mode: EventHandler<()>,
    on_theme_change: EventHandler<Theme>,
    on_settings: EventHandler<()>,
    client: ReadSignal<Option<Arc<OpenRouterClient>>>,
) -> Element {
    let theme_val = *theme.read();
    let is_dark = theme_val.is_dark();
    let mut dropdown_open = use_signal(|| false);
    let mut credits_dropdown_open = use_signal(|| false);
    let mut credits = use_signal(|| None::<CreditsData>);
    let mut is_loading_credits = use_signal(|| false);
    let mut has_fetched_credits = use_signal(|| false);

    // Fetch credits when dropdown opens for the first time
    use_effect(move || {
        if *credits_dropdown_open.read() && !*has_fetched_credits.read() {
            if let Some(client_ref) = client().clone() {
                has_fetched_credits.set(true);
                is_loading_credits.set(true);
                spawn(async move {
                    match client_ref.fetch_credits().await {
                        Ok(data) => {
                            credits.set(Some(data));
                        }
                        Err(e) => {
                            eprintln!("Failed to fetch credits: {}", e);
                        }
                    }
                    is_loading_credits.set(false);
                });
            }
        }
    });

    // Handler for refreshing credits
    let refresh_credits = move |_| {
        if let Some(client_ref) = client().clone() {
            is_loading_credits.set(true);
            spawn(async move {
                match client_ref.fetch_credits().await {
                    Ok(data) => {
                        credits.set(Some(data));
                    }
                    Err(e) => {
                        eprintln!("Failed to refresh credits: {}", e);
                    }
                }
                is_loading_credits.set(false);
            });
        }
    };

    let available_themes = if is_dark {
        Theme::dark_themes()
    } else {
        Theme::light_themes()
    };

    rsx! {
        header {
            class: "sticky top-0 z-30 before:absolute before:inset-0 before:backdrop-blur-md before:bg-[var(--color-base-200)]/90 before:-z-10 after:absolute after:h-px after:inset-x-0 after:top-full after:border-[var(--color-base-300)] after:-z-10",

            div {
                class: "px-4 sm:px-6 lg:px-8",

                div {
                    class: "flex items-center justify-between h-16",

                    // Left side - Title/Logo
                    div {
                        class: "flex items-center gap-2",

                        a {
                            class: "flex items-center gap-2",
                            href: "#",
                            span {
                                class: "text-xl font-bold text-[var(--color-base-content)]",
                                "GTLLM"
                            }
                        }
                    }

                    // Right side - Controls
                    div {
                        class: "flex items-center gap-3",

                        // Credits dropdown (if client is available)
                        if client().is_some() {
                            div {
                                class: "relative",

                                // Credits button
                                button {
                                    onclick: move |_| credits_dropdown_open.set(!credits_dropdown_open()),
                                    class: "px-3 py-1.5 rounded-lg bg-[var(--color-base-300)] text-[var(--color-base-content)] text-sm font-medium border border-[var(--color-base-300)] hover:bg-[var(--color-base-300)]/80 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-all cursor-pointer flex items-center gap-2",

                                    if let Some(credits_data) = credits() {
                                        "{credits_data.remaining_formatted()}"
                                    } else {
                                        "Loading..."
                                    }

                                    span {
                                        class: "text-[10px] opacity-50",
                                        if *credits_dropdown_open.read() { "▲" } else { "▼" }
                                    }
                                }

                                // Credits dropdown menu
                                if *credits_dropdown_open.read() {
                                    div {
                                        class: "absolute right-0 mt-2 w-56 bg-[var(--color-base-200)] border border-[var(--color-base-300)] rounded-lg shadow-lg z-50 overflow-hidden",

                                        // Credits info display
                                        if let Some(credits_data) = credits() {
                                            div {
                                                class: "p-3 space-y-2 border-b border-[var(--color-base-300)]",

                                                div {
                                                    class: "flex justify-between text-xs",
                                                    span {
                                                        class: "text-[var(--color-base-content)]/70",
                                                        "Total Credits:"
                                                    }
                                                    span {
                                                        class: "font-medium text-[var(--color-base-content)]",
                                                        "${credits_data.total_credits:.2}"
                                                    }
                                                }

                                                div {
                                                    class: "flex justify-between text-xs",
                                                    span {
                                                        class: "text-[var(--color-base-content)]/70",
                                                        "Total Usage:"
                                                    }
                                                    span {
                                                        class: "font-medium text-[var(--color-base-content)]",
                                                        "${credits_data.total_usage:.2}"
                                                    }
                                                }

                                                div {
                                                    class: "flex justify-between text-xs pt-1 border-t border-[var(--color-base-300)]/50",
                                                    span {
                                                        class: "text-[var(--color-base-content)]",
                                                        "Remaining:"
                                                    }
                                                    span {
                                                        class: "font-bold text-[var(--color-primary)]",
                                                        "{credits_data.remaining_formatted()}"
                                                    }
                                                }
                                            }
                                        }

                                        // Refresh button
                                        button {
                                            onclick: refresh_credits,
                                            class: "w-full px-3 py-2 text-left text-sm hover:bg-[var(--color-base-300)] transition-colors flex items-center gap-2",
                                            disabled: *is_loading_credits.read(),

                                            img {
                                                src: asset!("/assets/refresh.svg"),
                                                class: "w-4 h-4",
                                                class: if *is_loading_credits.read() { "animate-spin" } else { "" },
                                                alt: "Refresh"
                                            }

                                            span {
                                                class: "text-[var(--color-base-content)]",
                                                if *is_loading_credits.read() {
                                                    "Refreshing..."
                                                } else {
                                                    "Refresh Credits"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Theme dropdown (custom)
                        div {
                            class: "relative",

                            // Dropdown button
                            button {
                                onclick: move |_| dropdown_open.set(!dropdown_open()),
                                class: "px-3 py-1.5 rounded-lg bg-[var(--color-base-300)] text-[var(--color-base-content)] text-sm font-medium border border-[var(--color-base-300)] hover:bg-[var(--color-base-300)]/80 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-all cursor-pointer flex items-center gap-2",
                                "{theme_val.name()}"
                                span {
                                    class: "text-[10px] opacity-50",
                                    if *dropdown_open.read() { "▲" } else { "▼" }
                                }
                            }

                            // Dropdown menu
                            if *dropdown_open.read() {
                                div {
                                    class: "absolute right-0 mt-2 w-32 bg-[var(--color-base-200)] border border-[var(--color-base-300)] rounded-lg shadow-lg z-50 overflow-hidden",

                                    for available_theme in available_themes.iter() {
                                        {
                                            let theme_to_select = *available_theme;
                                            let is_selected = theme_to_select == theme_val;

                                            rsx! {
                                                button {
                                                    key: "{available_theme.data_theme()}",
                                                    onclick: move |_| {
                                                        on_theme_change.call(theme_to_select);
                                                        dropdown_open.set(false);
                                                    },
                                                    class: "w-full px-3 py-2 text-left text-sm hover:bg-[var(--color-base-300)] transition-colors",
                                                    class: if is_selected {
                                                        "bg-[var(--color-primary)]/10 text-[var(--color-base-content)] font-semibold"
                                                    } else {
                                                        "text-[var(--color-base-content)]"
                                                    },
                                                    "{available_theme.name()}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Dark/Light mode toggle
                        button {
                            onclick: move |_| on_toggle_mode.call(()),
                            class: "w-8 h-8 flex items-center justify-center hover:bg-[var(--color-base-300)]/50 rounded-full transition-colors",
                            title: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },

                            if is_dark {
                                img {
                                    src: asset!("/assets/light.svg"),
                                    class: "w-4 h-4",
                                    alt: "Light mode"
                                }
                            } else {
                                img {
                                    src: asset!("/assets/dark.svg"),
                                    class: "w-4 h-4",
                                    alt: "Dark mode"
                                }
                            }
                        }

                        // Settings button
                        button {
                            onclick: move |_| on_settings.call(()),
                            class: "w-8 h-8 flex items-center justify-center hover:bg-[var(--color-base-300)]/50 rounded-full transition-colors",
                            title: "Settings",
                            img {
                                src: asset!("/assets/settings.svg"),
                                class: "w-4 h-4",
                                alt: "Settings"
                            }
                        }
                    }
                }
            }
        }
    }
}
