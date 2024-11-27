# NOVA EGUI

Library to be able to use [egui](https://github.com/emilk/egui) with [nova](https://github.com/17cupsofcoffee/nova).

# Example

```rs

fn main() {
    let mut app = App::new("Your game", 1280, 720, 60.0);
    let mut state = YourState::new(&app);

    app.run(&mut state);
}

struct YourState {
    // ...
    egui: nova_egui::NovaEguiCtx,
}


impl YourState {
    fn new(app: &App) -> Self {
        Self {
            // ...

            // Initialize the nova egui context
            egui: nova_egui::NovaEguiCtx::new(app),
        }

    }
}


impl EventHandler for YourState {
    fn event(&mut self, app: &mut App, event: nova::input::Event) {
        // Propagate any events to egui
        self.egui.event(&event);
        // ..
    }


    fn update(&mut self, app: &mut App) {
        // Notify egui that an update occured
        self.egui.update(app);
        // ..
    }


    fn draw(&mut self, app: &mut App) {
        // ..

        // At the end, add your egui logic:

        self.egui.render(app, |ctx| {
            // ctx can be used with the following egui types to get a ui:
            // SidePanel, TopBottomPanel, CentralPanel, Window or Area
            egui::Window::new("Example").show(ctx, |app, ui| {
                // `app` is a mutable reference to the `app` you passed in
                // you also have mutable access to `self` here (except for `self.egui`)

                // `ui` can be used to render egui widgets
                ui.label("Hello world!");
            });
        });
    }
}

```

Hobby project, use at own risk, etc etc

## License

Licensed under [BBHL](https://lifning.info/BBHL)
