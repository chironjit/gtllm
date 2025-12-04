# GTLLM Project Overview

**GTLLM** (Game Theory LLM) is a Rust-based desktop application built with **Dioxus 0.7** that implements a multi-LLM chat system inspired by game theory concepts. Uses OpenRouter APIs for inferencing

### Key Features

**5 Chat Modes:**
1. **Standard** - Normal single (or multiple) LLMs in individual chats
2. **PvP** - 2 LLMs compete with 1 moderator judging
3. **Collaborative** - Multiple LLMs work together to agree on the best solution
4. **Competitive** - LLMs propose solutions and vote on the best one (can't vote for their own)
5. **LLM's Choice** - The LLMs decide whether to collaborate or compete

### Tech Stack

- **Framework**: Dioxus 0.7
- **Styling**: Tailwind CSS
- **HTTP Client**: reqwest 
- **Async Runtime**: Tokio
- **Serialization**: serde/serde_json

### Project Structure
```
gtllm/
├── src
│   ├── main.rs
│   ├── components
│   │   ├── mod.rs                  
│   │   ├── header.rs                   # Header component
│   │   ├── sidebar.rs                  # Sidebar component
│   │   └── modes                       # Main section - mainly chat modes but also includes settings page
│   │       ├── mod.rs             
│   │       ├── collaborative.rs
│   │       ├── standard.rs
│   │       ├── competitive.rs
│   │       ├── choice.rs
│   │       ├── new_chat.rs
│   │       ├── pvp.rs
│   │       ├── settings.rs
│   │       └── common                  # Common components used by the modes
│   │           ├── mod.rs
│   │           ├── input.rs
│   │           ├── model_selector.rs
│   │           ├── chat.rs
│   │           └── selection.rs
│   └── utils
│       ├── mod.rs
│       ├── openrouter.rs               # OpenRouter API client
│       ├── settings.rs                 # Settings persistence
│       ├── theme.rs                    # Theme management
│       ├── types.rs                    # Type definitions
│       └── formatting.rs               # Output formatting
├── guides                              # Guides to help with code generation
│   ├── dioxus
│   │   └── DIOXUS_COMPONENTS.md
│   └── openrouter
│       ├── API_REF.md
│       ├── EMBEDDINGS.md
│       ├── OTHERS.md
│       └── STREAMING.md
├── AGENTS.md                           # Agent / LLM instructions
├── assets                              # Assets incl icons, images & generated files
├── Cargo.lock
├── Cargo.toml
├── Dioxus.toml                           
├── README.md
└── tailwind.css
```

### Configuration

- **OpenRouter API** for LLM access (requires API key in settings)
- Settings stored persistently using `dirs` crate
- Support multiple themes (Dracula, Winter, etc.)
- Customizable input settings (Ctrl+Enter to submit)

## Instructions
1. This is a Dioxus 0.7 and Tailwind 4.0 app. Read the notes in this document on how to use the latest Dioxus version
2. It is important to use the right code in the right place
  - All CSS should be in TailwindCSS ( version 4.0 and above) classes only
  - All theme-related items in `src/utils/theme.rs` and `tailwind.css`
  - All type definition in `src/utils/types.rs`
  - All generic input and output formatting related items in `src/utils/formatting.rs`
  - All OpenRouter API relaten items in `src/utils/openrouter.rs`
  - All settings persistence related code in `src/utils/settings.rs`
  - Main section code in `src/components/modes/`. 
  - Any common subcomponents for the modes should be in the `src/components/modes/common`. Before creating any component for any mode, check this folder
3. Use the examples, documentation and code in the `guides` folder to help you witn your code

---

## Dioxus Notes
You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant. Dioxus 0.7 changes every api in dioxus. Only use this up to date documentation. `cx`, `Scope`, and `use_state` are gone

Provide concise code examples with detailed descriptions

### Dioxus Dependency

You can add Dioxus to your `Cargo.toml` like this:

```toml
[dependencies]
dioxus = { version = "0.7.1" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

### Launching your application

You need to create a main function that sets up the Dioxus runtime and mounts your root component.

```rust
use dioxus::prelude::*;

fn main() {
	dioxus::launch(App);
}

#[component]
fn App() -> Element {
	rsx! { "Hello, Dioxus!" }
}
```

Then serve with `dx serve`:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

### UI with RSX

```rust
rsx! {
	div {
		class: "container", // Attribute
		color: "red", // Inline styles
		width: if condition { "100%" }, // Conditional attributes
		"Hello, Dioxus!"
	}
	// Prefer loops over iterators
	for i in 0..5 {
		div { "{i}" } // use elements or components directly in loops
	}
	if condition {
		div { "Condition is true!" } // use elements or components directly in conditionals
	}

	{children} // Expressions are wrapped in brace
	{(0..5).map(|i| rsx! { span { "Item {i}" } })} // Iterators must be wrapped in braces
}
```

### Assets

The asset macro can be used to link to local files to use in your project. All links start with `/` and are relative to the root of your project.

```rust
rsx! {
	img {
		src: asset!("/assets/image.png"),
		alt: "An image",
	}
}
```

#### Styles

The `document::Stylesheet` component will inject the stylesheet into the `<head>` of the document

```rust
rsx! {
	document::Stylesheet {
		href: asset!("/assets/styles.css"),
	}
}
```

### Components

Components are the building blocks of apps

* Component are functions annotated with the `#[component]` macro.
* The function name must start with a capital letter or contain an underscore.
* A component re-renders only under two conditions:
	1.  Its props change (as determined by `PartialEq`).
	2.  An internal reactive state it depends on is updated.

```rust
#[component]
fn Input(mut value: Signal<String>) -> Element {
	rsx! {
		input {
            value,
			oninput: move |e| {
				*value.write() = e.value();
			},
			onkeydown: move |e| {
				if e.key() == Key::Enter {
					value.write().clear();
				}
			},
		}
	}
}
```

Each component accepts function arguments (props)

* Props must be owned values, not references. Use `String` and `Vec<T>` instead of `&str` or `&[T]`.
* Props must implement `PartialEq` and `Clone`.
* To make props reactive and copy, you can wrap the type in `ReadOnlySignal`. Any reactive state like memos and resources that read `ReadOnlySignal` props will automatically re-run when the prop changes.

### State

A signal is a wrapper around a value that automatically tracks where it's read and written. Changing a signal's value causes code that relies on the signal to rerun.

#### Local State

The `use_signal` hook creates state that is local to a single component. You can call the signal like a function (e.g. `my_signal()`) to clone the value, or use `.read()` to get a reference. `.write()` gets a mutable reference to the value.

Use `use_memo` to create a memoized value that recalculates when its dependencies change. Memos are useful for expensive calculations that you don't want to repeat unnecessarily.

```rust
#[component]
fn Counter() -> Element {
	let mut count = use_signal(|| 0);
	let mut doubled = use_memo(move || count() * 2); // doubled will re-run when count changes because it reads the signal

	rsx! {
		h1 { "Count: {count}" } // Counter will re-render when count changes because it reads the signal
		h2 { "Doubled: {doubled}" }
		button {
			onclick: move |_| *count.write() += 1, // Writing to the signal rerenders Counter
			"Increment"
		}
		button {
			onclick: move |_| count.with_mut(|count| *count += 1), // use with_mut to mutate the signal
			"Increment with with_mut"
		}
	}
}
```

#### Context API

The Context API allows you to share state down the component tree. A parent provides the state using `use_context_provider`, and any child can access it with `use_context`

```rust
#[component]
fn App() -> Element {
	let mut theme = use_signal(|| "light".to_string());
	use_context_provider(|| theme); // Provide a type to children
	rsx! { Child {} }
}

#[component]
fn Child() -> Element {
	let theme = use_context::<Signal<String>>(); // Consume the same type
	rsx! {
		div {
			"Current theme: {theme}"
		}
	}
}
```

### Async

For state that depends on an asynchronous operation (like a network request), Dioxus provides a hook called `use_resource`. This hook manages the lifecycle of the async task and provides the result to your component.

* The `use_resource` hook takes an `async` closure. It re-runs this closure whenever any signals it depends on (reads) are updated
* The `Resource` object returned can be in several states when read:
1. `None` if the resource is still loading
2. `Some(value)` if the resource has successfully loaded

```rust
let mut dog = use_resource(move || async move {
	// api request
});

match dog() {
	Some(dog_info) => rsx! { Dog { dog_info } },
	None => rsx! { "Loading..." },
}
```

### Routing

All possible routes are defined in a single Rust `enum` that derives `Routable`. Each variant represents a route and is annotated with `#[route("/path")]`. Dynamic Segments can capture parts of the URL path as parameters by using `:name` in the route string. These become fields in the enum variant.

The `Router<Route> {}` component is the entry point that manages rendering the correct component for the current URL.

You can use the `#[layout(NavBar)]` to create a layout shared between pages and place an `Outlet<Route> {}` inside your layout component. The child routes will be rendered in the outlet.

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
	#[layout(NavBar)] // This will use NavBar as the layout for all routes
		#[route("/")]
		Home {},
		#[route("/blog/:id")] // Dynamic segment
		BlogPost { id: i32 },
}

#[component]
fn NavBar() -> Element {
	rsx! {
		a { href: "/", "Home" }
		Outlet<Route> {} // Renders Home or BlogPost
	}
}

#[component]
fn App() -> Element {
	rsx! { Router::<Route> {} }
}
```

```toml
dioxus = { version = "0.7.1", features = ["router"] }
```

### Fullstack

Fullstack enables server rendering and ipc calls. It uses Cargo features (`server` and a client feature like `web`) to split the code into a server and client binaries.

```toml
dioxus = { version = "0.7.1", features = ["fullstack"] }
```

#### Server Functions

Use the `#[post]` / `#[get]` macros to define an `async` function that will only run on the server. On the server, this macro generates an API endpoint. On the client, it generates a function that makes an HTTP request to that endpoint.

```rust
#[post("/api/double/:path/&query")]
async fn double_server(number: i32, path: String, query: i32) -> Result<i32, ServerFnError> {
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	Ok(number * 2)
}
```

#### Hydration

Hydration is the process of making a server-rendered HTML page interactive on the client. The server sends the initial HTML, and then the client-side runs, attaches event listeners, and takes control of future rendering.

##### Errors
The initial UI rendered by the component on the client must be identical to the UI rendered on the server.

* Use the `use_server_future` hook instead of `use_resource`. It runs the future on the server, serializes the result, and sends it to the client, ensuring the client has the data immediately for its first render.
* Any code that relies on browser-specific APIs (like accessing `localStorage`) must be run *after* hydration. Place this code inside a `use_effect` hook.
