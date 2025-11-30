# GTLLM
Inspired by Karpathy's council llm and veritaseum's game theory video. Basically there are 3 modes:
1) PvP - 2 bots 1 moderator
2) Collaborative - pick however many but all must jointly agree on the best solution
3) Competitive - all must vote for the one that is the best, can't vote for theirs
4) LLM's option - collaborate or compete



Future ideas:
1) Punishment / retaliation
2) Alliances

## Warning
Please note that any choice you make to add more endpoints will automatically add on more cost to each call. It will also add on cost in tokens, so be mindful that this can bun through tokens and context very fast.


## Download and run
Linux: Download appimage and make it executable. Then start by double clicking it. (or use Gear Lever app to add it to your app menu)

Windows: (Coming soon)

MacOS: (Coming soon)

Note: You will need to have your own Openrouter API key to use this app. Use openrouter's api and set it in the settings section

## Development
The app has been developed using Dioxus. The following is the dioxus guide to getting started

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # The entrypoint for the app.
│  ├─ components/
│  │  ├─ mod.rs # Defines the components module
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Automatic Tailwind (Dioxus 0.7+)

As of Dioxus 0.7, there no longer is a need to manually install tailwind. Simply `dx serve` and you're good to go!

Automatic tailwind is supported by checking for a file called `tailwind.css` in your app's manifest directory (next to Cargo.toml). To customize the file, use the dioxus.toml:

```toml
[application]
tailwind_input = "my.css"
tailwind_output = "assets/out.css"
```

### Tailwind Manual Install

To use tailwind plugins or manually customize tailwind, you can can install the Tailwind CLI and use it directly.

1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the Tailwind CSS CLI: https://tailwindcss.com/docs/installation/tailwind-cli
3. Run the following command in the root of the project to start the Tailwind CSS compiler:

```bash
npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```
