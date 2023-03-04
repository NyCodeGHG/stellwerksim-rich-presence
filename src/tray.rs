use tao::{
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayEvent, TrayIconBuilder,
};

pub struct TrayActor {}

impl TrayActor {
    pub fn spawn() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");
        let icon = load_icon(std::path::Path::new(path));
        let mut event_loop = EventLoop::new();
        let menu = Menu::new();
        let quit = MenuItem::new("Quit", true, None);
        menu.append_items(&[
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some(env!("CARGO_PKG_NAME").to_string()),
                    copyright: Some("Copyright (c) 2023 Marie Ramlow".to_string()),
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    license: Some(env!("CARGO_PKG_LICENSE").to_string()),
                    ..Default::default()
                }),
            ),
            &quit,
        ]);

        let mut tray_icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("StellwerkSim Rich Presence")
                .with_icon(icon)
                .build()
                .unwrap(),
        );

        let menu_channel = MenuEvent::receiver();
        let tray_channel = TrayEvent::receiver();

        event_loop.run_return(move |_, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            if let Ok(event) = menu_channel.try_recv() {
                if event.id == quit.id() {
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                }
                println!("{event:#?}")
            }

            if let Ok(event) = tray_channel.try_recv() {
                println!("{event:#?}")
            }
        });
    }
}

fn load_icon(path: &std::path::Path) -> tray_icon::icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to open icon")
}
