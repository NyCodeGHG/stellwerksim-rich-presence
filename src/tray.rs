use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayEvent, TrayIconBuilder,
};
use winit::event_loop::{ControlFlow, EventLoop};

pub struct TrayActor {}

impl TrayActor {
    pub fn spawn() {
        let event_loop = EventLoop::new();
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
                .build()
                .unwrap(),
        );

        let menu_channel = MenuEvent::receiver();
        let tray_channel = TrayEvent::receiver();

        event_loop.run(move |_, _, control_flow| {
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
        })
    }
}
