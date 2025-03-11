<!--toc:start-->
- [egui w/ eframe - local ~~and wasm~~ builds](#egui-w-eframe-local-and-wasm-builds)
- [live examples](#live-examples)
- [core ideas](#core-ideas)
- [egui](#egui)
- [eframe](#eframe)
  - [Core Trait](#core-trait)
  - [Fancy-Fancy (standard) Style of Use](#fancy-fancy-standard-style-of-use)
<!--toc:end-->

# egui w/ eframe - local ~~and wasm~~ builds

# live examples

- core: [www.egui.rs/#demo](https://www.egui.rs/#demo)
- etable: [rerun-io.github.io/egui_table/](https://rerun-io.github.io/egui_table/)
- eplot: [emilk.github.io/egui_plot/](https://emilk.github.io/egui_plot/)
- lmerlin: [lucasmerlin.github.io/hello_egui](https://lucasmerlin.github.io/hello_egui/#/example/color_sort_vertical)

# core ideas

```
_______________               _____________                ________                _____________
| "Framework" |  ---------->  | "Context" |  ----------->  | "Ui" |  ----------->  | "Widgets" |
---------------               -------------                --------                -------------
aka                           aka                          aka                     aka
'system hookup'               'egui context'               'specific region'       'stuff'
'umbilical'                   'deserialized ctx'           'canvas corner'
'outer space interface'
'post-rust translator'

```
 ...             |             ...             |             ...              |          ... |
| "**Framework**" |        "**Context**"        |           "**Ui**"           | "**Widgets**" |
| :-------------- | :-------------------------: | :--------------------------: | ------------: |
| eframe          | eframe::`run_simple_native` | (Centra/Side/TopBottom)Panel | ui.horizontal |
| bevy_egui       |    eframe::`run_native`     |          ScrollArea          |      ui.label |
|                 |                             |            Window            |     ui.slider |
| ...             |             ...             |             Area             |    ui.button  |
| ...             |             ...             |             ...              |           ... |

- [Context](file:///Users/esl/coding_dirs/rust/egui_xp/target/doc/egui/struct.Context.html)
  - RefCounted (cheap to clone, shared mutable data)
  - RwLock protected data (closure-taking methods deal with this automatically)
  - recursive locking of context (within a closure) _NOT_ allowed

# egui

- **what**: with*in* Rust library for doing GUI-like data actions
  - The main thing the programmer works with, but that gets and gives data via an umbilical "framework" that can translate to the machine/systems language that generates images, provides inputs, etc.

```markdown
To create a GUI using egui you first need a **Context** (by convention referred to by **ctx**). Then you add a Window or a SidePanel to get a **Ui**, which is what you’ll be using to add all the buttons and labels that you need.
```

# eframe

- **what**: umbilical that connects egui to multiple platforms. ("framework" in egui lang)
- **why**: 'general purpose' connector to both "native" (OS) and "web" (WASM) platforms
- **how**: implement _eframe_::`App` trait (define `update`) and then use _eframe_::`run(_simple)_native`
  - _wasm_: look at eframe template for additional details on building for wasm and deploying
  - _note_: `App` can be implemented on an _empty struct_. It is common to persist information across frames _via_ that struct, but not necessary. (This is particularly helpful when experimenting or when dropping in an exploratory UI to an existing program.)

## Core Trait
```rust
/// The interface by which `eframe` runs our code.
pub trait App {
    // Required method
    fn update(&mut self, ctx: &Context, frame: &mut Frame);

    // Provided methods
    fn save(&mut self, _storage: &mut dyn Storage) { ... }
    fn on_exit(&mut self, _gl: Option<&Context>) { ... }
    fn auto_save_interval(&self) -> Duration { ... }
    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] { ... }
    fn persist_egui_memory(&self) -> bool { ... }
    fn raw_input_hook(&mut self, _ctx: &Context, _raw_input: &mut RawInput) { ... }
}
```

## Fancy-Fancy (standard) Style of Use
```rust
use eframe::egui;

// define eframe options (e.g. where to save data); `::default()` works well to get startedpart
// Then `run_native` with a Boxed closure that returns a Result<Boxed App> (`AppCreator` as type alias)
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("AppName", native_options, Box::new(|cc| Ok(Box::new(AppStruct::new(cc)))))
}

#[derive(Default)]
struct AppStruct {
        // Optionally add fields and data here
        // Fields useful for holding data across frames and for persisting app-specific data
}

impl AppStruct {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for AppStruct {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       egui::Panel::default().show(ctx, |ui| {
       });
   }
}
```

# egui demo to eframe app

- ignore: `crate::Demo::name()`
- change: `impl crate::Demo for xxxx`~~> `impl eframe::App`
- ignore: `fn name()`
