# Dioxus Components guide

## Toast

### Install
```
dx components add toast
```

### Use
```
use crate::components::button::component::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;

#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { ToastButton {} }
    }
}

#[component]
fn ToastButton() -> Element {
    let toast_api = use_toast();

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            onclick: move |_| {
                toast_api
                    .info(
                        "Custom Toast".to_string(),
                        ToastOptions::new()
                            .description("Some info you need")
                            .duration(Duration::from_secs(60))
                            .permanent(false),
                    );
            },
            "Info (60s)"
        }
    }
}

```

### Component structure
```
// The Toast provider provides the toast context to its children and handler rendering any toasts that are sent.
ToastProvider {
    // Any child component can consume the toast context and send a toast to be rendered.
    button {
        onclick: |event: MouseEvent| {
            // Consume the toast context to send a toast.
            let toast_api = consume_toast();
            toast_api
                .error(
                    "Critical Error".to_string(),
                    ToastOptions::new()
                        .description("Some info you need")
                        .duration(Duration::from_secs(60))
                        .permanent(false),
                );
        },
        "Show Toast"
    }
}
```

## Card
### Install
```
dx components add card
```

### Use
```
use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::label::Label;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Card { style: "width: 100%; max-width: 24rem;",
            CardHeader {
                CardTitle { "Login to your account" }
                CardDescription { "Enter your email below to login to your account" }
                CardAction {
                    Button { variant: ButtonVariant::Ghost, "Sign Up" }
                }
            }
            CardContent {
                form {
                    div { style: "display: flex; flex-direction: column; gap: 1.5rem;",
                        div { style: "display: grid; gap: 0.5rem;",
                            Label { html_for: "email", "Email" }
                            Input {
                                id: "email",
                                r#type: "email",
                                placeholder: "m@example.com",
                            }
                        }
                        div { style: "display: grid; gap: 0.5rem;",
                            div { style: "display: flex; align-items: center;",
                                Label { html_for: "password", "Password" }
                                a {
                                    href: "#",
                                    style: "margin-left: auto; font-size: 0.875rem; color: var(--secondary-color-5); text-decoration: underline; text-underline-offset: 4px;",
                                    "Forgot your password?"
                                }
                            }
                            Input { id: "password", r#type: "password" }
                        }
                    }
                }
            }
            CardFooter { style: "flex-direction: column; gap: 0.5rem;",
                Button { r#type: "submit", style: "width: 100%;", "Login" }
                Button { variant: ButtonVariant::Outline, style: "width: 100%;", "Login with Google" }
            }
        }
    }
}

```

### Component Structure
```
// The Card component must wrap all card elements.
Card {
    // CardHeader contains the title, description, and optional action.
    CardHeader {
        // CardTitle displays the main heading.
        CardTitle { "Card Title" }
        // CardDescription provides supporting text.
        CardDescription { "Card description goes here." }
        // CardAction positions action elements (e.g., buttons) in the header.
        CardAction {
            Button { "Action" }
        }
    }
    // CardContent holds the main body content.
    CardContent {
        p { "Main content of the card." }
    }
    // CardFooter contains footer actions or information.
    CardFooter {
        Button { "Submit" }
    }
}
```

## Skeleton
### Install
```
dx components add skeleton
```
### Use
```
use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 2rem;",
            SkeletonInfoDemo {}
            SkeletonCardDemo {}
        }
    }
}

#[component]
fn SkeletonInfoDemo() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 1rem;",
            Skeleton { style: "width: 3rem; height: 3rem; border-radius: 50%;" }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                Skeleton { style: "width: 11.625rem; height: 1rem;" }
                Skeleton { style: "width: 8.5rem; height: 1rem;" }
            }
        }
    }
}

#[component]
fn SkeletonCardDemo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem;",
            Skeleton { style: "width: 15rem; height: 8rem; border-radius: 0.75rem;" }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                Skeleton { style: "width: 15.625rem; height: 1rem;" }
                Skeleton { style: "width: 12.5rem; height: 1rem;" }
            }
        }
    }
}

```
### Component Structure
```
Skeleton {
    // Accepts all GlobalAttributes, commonly used with style for sizing
    style: "width: 15rem; height: 1rem;",
}

```

## Scrollable Area
### Install
```
dx components add scroll_area
```
### Use
```
use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::scroll_area::ScrollDirection;

#[component]
pub fn Demo() -> Element {
    rsx! {

        ScrollArea {
            width: "10em",
            height: "10em",
            border: "1px solid var(--primary-color-6)",
            border_radius: "0.5em",
            padding: "0 1em 1em 1em",
            direction: ScrollDirection::Vertical,
            tabindex: "0",
            div { class: "scroll-content",
                for i in 1..=20 {
                    p { "Scrollable content item {i}" }
                }
            }
        }
    }
}
```
### Component Structure
```
// The ScrollArea component wraps all scrollable content.
ScrollArea {
    // The direction in which the scroll area can scroll. Can be one of Horizontal, Vertical, or Both.
    scroll_direction: ScrollDirection::Vertical,
    // The content of the scrollable area
    {children}
}
```

## Tooltip
### Install
```
dx components add tooltip
```
### Use
```
use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::ContentSide;

#[component]
pub fn Demo() -> Element {
    rsx! {

        Tooltip {
            TooltipTrigger { "Rich content" }
            TooltipContent { side: ContentSide::Left, style: "width: 200px;",
                h4 { style: "margin-top: 0; margin-bottom: 8px;", "Tooltip title" }
                p { style: "margin: 0;", "This tooltip contains rich HTML content with styling." }
            }
        }
    }
}

```
### Component Structure
```
// The Tooltip component wraps the trigger element and the content that will be displayed on hover.
Tooltip {
    // The TooltipTrigger contains the elements that will trigger the tooltip to display when hovered over.
    TooltipTrigger {
        // The elements that will trigger the tooltip when hovered over.
        {children}
    }
    // The TooltipContent contains the content that will be displayed when the user hovers over the trigger.
    TooltipContent {
        // The side of the TooltipTrigger where the content will be displayed. Can be one of Top, Right, Bottom, or Left.
        side: ContentSide::Top,
        // The alignment of the TooltipContent relative to the TooltipTrigger. Can be one of Start, Center, or End.
        align: ContentAlign::Center,
        // The content of the tooltip, which can include text, images, or any other elements.
        {children}
    }
}
```

## Dialog
### Install
```
dx components add dialog
```
### Use
```
use crate::components::button::component::Button;

use super::super::component::{DialogContent, DialogDescription, DialogRoot, DialogTitle};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            style: "margin-bottom: 1.5rem;",
            onclick: move |_| open.set(true),
            "Show Dialog"
        }
        DialogRoot { open: open(), on_open_change: move |v| open.set(v),
            DialogContent {
                button {
                    class: "dialog-close",
                    r#type: "button",
                    aria_label: "Close",
                    tabindex: if open() { "0" } else { "-1" },
                    onclick: move |_| open.set(false),
                    "×"
                }
                DialogTitle { "Item information" }
                DialogDescription { "Here is some additional information about the item." }
            }
        }
    }
}
```
### Component Structure
```
// The dialog component must wrap all dialog elements.
Dialog {
    // The open prop determines if the dialog is currently open or closed.
    open: open(),
    // The dialog title defines the heading of the dialog.
    DialogTitle {
        "Item information"
    }
    // The dialog description provides additional information about the dialog.
    DialogDescription {
        "Here is some additional information about the item."
    }
}
```

## Sheet
### Install
```
dx components add sheet
```
### Use
```
use crate::components::{
    button::{Button, ButtonVariant},
    input::Input,
    label::Label,
    sheet::{
        Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
        SheetTitle,
    },
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut side = use_signal(|| SheetSide::Right);

    let open_sheet = move |s: SheetSide| {
        move |_| {
            side.set(s);
            open.set(true);
        }
    };

    rsx! {
        div { display: "flex", gap: "0.5rem",
            Button { variant: ButtonVariant::Outline, onclick: open_sheet(SheetSide::Top), "Top" }
            Button { variant: ButtonVariant::Outline, onclick: open_sheet(SheetSide::Right), "Right" }
            Button { variant: ButtonVariant::Outline, onclick: open_sheet(SheetSide::Bottom), "Bottom" }
            Button { variant: ButtonVariant::Outline, onclick: open_sheet(SheetSide::Left), "Left" }
        }
        Sheet { open: open(), on_open_change: move |v| open.set(v),
            SheetContent { side: side(),
                SheetHeader {
                    SheetTitle { "Sheet Title" }
                    SheetDescription { "Sheet description goes here." }
                }

                div {
                    display: "grid",
                    flex: "1 1 0%",
                    grid_auto_rows: "min-content",
                    gap: "1.5rem",
                    padding: "0 1rem",
                    div { display: "grid", gap: "0.75rem",
                        Label { html_for: "sheet-demo-name", "Name" }
                        Input {
                            id: "sheet-demo-name",
                            initial_value: "Dioxus",
                        }
                    }
                    div { display: "grid", gap: "0.75rem",
                        Label { html_for: "sheet-demo-username", "Username" }
                        Input {
                            id: "sheet-demo-username",
                            initial_value: "@dioxus",
                        }
                    }
                }

                SheetFooter {
                    Button { "Save changes" }
                    SheetClose {
                        r#as: |attributes| rsx! {
                            Button { variant: ButtonVariant::Outline, attributes, "Cancel" }
                        },
                    }
                }
            }
        }
    }
}
```

### Component Structure
```
// The sheet component must wrap all sheet elements.
Sheet {
    // The open prop determines if the sheet is currently open or closed.
    open: open(),
    // SheetContent wraps the content and defines the side from which the sheet slides in.
    // Available sides: Top, Right (default), Bottom, Left.
    SheetContent {
        side: SheetSide::Right,
        // SheetHeader groups the title and description at the top.
        SheetHeader {
            // The sheet title defines the heading of the sheet.
            SheetTitle {
                "Edit Profile"
            }
            // The sheet description provides additional information about the sheet.
            SheetDescription {
                "Make changes to your profile here."
            }
        }
        // Add your main content here.
        // SheetFooter groups actions at the bottom.
        SheetFooter {
            // SheetClose can be used to close the sheet.
            SheetClose {
                "Close"
            }
        }
    }
}
```

### SheetClose with `as` prop
```
// Default: renders as <button>
SheetClose { "Close" }

// Custom element: attributes include the preset onclick handler
SheetClose {
    r#as: |attributes| rsx! {
        a { href: "#", ..attributes, "Go back" }
    }
}
```

## Hover
### Install
```
dx components add hover_card
```
### Use
```
use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::ContentSide;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "padding: 50px; display: flex; flex-direction: row; flex-wrap: wrap; gap: 40px; justify-content: center; align-items: center;",
            HoverCard {
                HoverCardTrigger {
                    i { "Dioxus" }
                }
                HoverCardContent { side: ContentSide::Bottom,
                    div { padding: "1rem",
                        "Dioxus is"
                        i { " the " }
                        "Rust framework for building fullstack web, desktop, and mobile apps. Iterate with live hotreloading, add server functions, and deploy in record time."
                    }
                }
            }
        }
    }
}
```
### Component Structure
```
// The HoverCard component wraps the trigger element and the content that will be displayed on hover.
HoverCard {
    // The HoverCardTrigger contains the elements that will trigger the hover card to display when hovered.
    HoverCardTrigger {
        // The elements that will trigger the hover card when hovered over.
        {children}
    }
    // The HoverCardContent contains the content that will be displayed when the user hovers over the trigger.
    HoverCardContent {
        // The side of the HoverCardTrigger where the content will be displayed. Can be one Top, Right, Bottom, or Left.
        side: ContentSide::Bottom,
        // The alignment of the HoverCardContent relative to the HoverCardTrigger. Can be one of Start, Center, or End.
        align: ContentAlign::Start,
        // The content of the hover card, which can include text, images, or any other elements.
        {children}
    }
}
```

## Collapsible
### Install
```
dx components add collapsible
```
### Use
```
use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Collapsible {
            CollapsibleTrigger {
                b { "Recent Activity" }
            }
            CollapsibleList {
                CollapsibleItem { "Added a new feature to the collapsible component" }
                CollapsibleContent {
                    CollapsibleItem { "Fixed a bug in the collapsible component" }
                    CollapsibleItem { "Updated the documentation for the collapsible component" }
                }
            }
        }
    }
}

```
### Component Structure
```
// The collapsible component must wrap all collapsible items.
Collapsible {
    // The trigger is used to expand or collapse the item.
    CollapsibleTrigger {}
    // The content that is shown when the item is expanded.
    CollapsibleContent {}
}
```

## Component
### Install
```

```
### Use
```

```
### Component Structure
```

```
