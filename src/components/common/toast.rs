use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

#[derive(Clone, Debug)]
pub struct ToastMessage {
    pub id: usize,
    pub message: String,
    pub toast_type: ToastType,
}

impl ToastMessage {
    fn bg_class(&self) -> &'static str {
        match self.toast_type {
            ToastType::Success => "bg-green-600/90",
            ToastType::Error => "bg-red-600/90",
            ToastType::Info => "bg-blue-600/90",
        }
    }

    fn icon(&self) -> &'static str {
        match self.toast_type {
            ToastType::Success => "✓",
            ToastType::Error => "✕",
            ToastType::Info => "ℹ",
        }
    }
}

static TOAST_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn add_toast(
    mut toasts: Signal<Vec<ToastMessage>>,
    toast_type: ToastType,
    message: String,
) {
    let id = TOAST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let toast = ToastMessage {
        id,
        message,
        toast_type,
    };
    toasts.write().push(toast);
}

#[component]
pub fn ToastContainer(toasts: Signal<Vec<ToastMessage>>) -> Element {
    let mut toasts_signal = toasts;

    // Auto-dismiss: schedule removal of each toast after 4 seconds
    {
        let current_toasts: Vec<ToastMessage> = toasts_signal.read().clone();
        for toast in &current_toasts {
            let toast_id = toast.id;
            let mut ts = toasts_signal;
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                ts.write().retain(|t| t.id != toast_id);
            });
        }
    }

    let toasts_list = toasts_signal.read();

    if toasts_list.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed bottom-4 right-4 z-[9999] flex flex-col gap-2 max-w-sm",

            for toast in toasts_list.iter() {
                {
                    let toast_id = toast.id;
                    let bg = toast.bg_class();
                    let icon = toast.icon();
                    let msg = toast.message.clone();

                    rsx! {
                        div {
                            key: "{toast_id}",
                            class: "flex items-center gap-2 px-4 py-3 rounded-lg shadow-lg text-white text-sm animate-fade-in {bg}",

                            span { class: "text-base font-bold", "{icon}" }
                            span { class: "flex-1", "{msg}" }

                            button {
                                class: "text-white/70 hover:text-white ml-2 text-lg leading-none",
                                onclick: move |_| {
                                    toasts_signal.write().retain(|t| t.id != toast_id);
                                },
                                "×"
                            }
                        }
                    }
                }
            }
        }
    }
}
