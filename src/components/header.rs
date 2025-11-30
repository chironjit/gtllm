use crate::utils::Theme;
use dioxus::prelude::*;

#[component]
pub fn Header(
    theme: Signal<Theme>,
    on_toggle_mode: EventHandler<()>,
    on_theme_change: EventHandler<Theme>,
    on_settings: EventHandler<()>,
) -> Element {
    let theme_val = *theme.read();
    let is_dark = theme_val.is_dark();

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

                        // Theme dropdown
                        select {
                            value: "{theme_val.data_theme()}",
                            onchange: move |evt| {
                                let selected_name = evt.value();
                                if let Some(new_theme) = Theme::all()
                                    .into_iter()
                                    .find(|t| t.data_theme() == selected_name)
                                {
                                    on_theme_change.call(new_theme);
                                }
                            },
                            class: "px-3 py-1.5 rounded-lg text-sm font-medium border-2 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] transition-all cursor-pointer",
                            style: "background-color: var(--color-base-300); color: var(--color-base-content); border-color: var(--color-base-300);",

                            for available_theme in available_themes.iter() {
                                option {
                                    key: "{available_theme.data_theme()}",
                                    value: "{available_theme.data_theme()}",
                                    selected: *available_theme == theme_val,
                                    style: "background-color: var(--color-base-200); color: var(--color-base-content);",
                                    "{available_theme.name()}"
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
