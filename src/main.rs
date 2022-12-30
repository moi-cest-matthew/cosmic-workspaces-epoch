use cctk::{
    cosmic_protocols::workspace::v1::client::zcosmic_workspace_handle_v1,
    sctk::shell::layer::{Anchor, KeyboardInteractivity, Layer},
    wayland_client::protocol::wl_output,
};
use iced::{
    event::wayland::{Event as WaylandEvent, OutputEvent},
    keyboard::KeyCode,
    sctk_settings::InitialSurface,
    Application, Command, Element, Subscription,
};
use iced_native::{
    command::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    window::Id as SurfaceId,
};
use iced_sctk::{
    application::SurfaceIdWrapper,
    commands::layer_surface::{destroy_layer_surface, get_layer_surface},
};
use std::{collections::HashMap, process};

mod wayland;

#[derive(Debug)]
enum Msg {
    WaylandEvent(WaylandEvent),
    Wayland(wayland::Event),
    Close,
    Closed(SurfaceIdWrapper),
}

#[derive(Debug)]
struct Workspace {
    name: String,
    img: Option<iced::widget::image::Handle>,
    handle: zcosmic_workspace_handle_v1::ZcosmicWorkspaceHandleV1,
    output: wl_output::WlOutput,
}

struct LayerSurface {
    output: wl_output::WlOutput,
    // Active workspace
    // windows in workspace
    // - for transitions, would need windows in more than one workspace
}

#[derive(Default)]
struct App {
    max_surface_id: usize,
    layer_surfaces: HashMap<SurfaceId, LayerSurface>,
    workspaces: Vec<Workspace>,
}

impl App {
    fn next_surface_id(&mut self) -> SurfaceId {
        self.max_surface_id += 1;
        SurfaceId::new(self.max_surface_id)
    }
}

impl Application for App {
    type Message = Msg;
    type Theme = cosmic::Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Msg>) {
        //(Self::default(), destroy_layer_surface(SurfaceId::new(0)))
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("cosmic-workspaces")
    }

    fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg::WaylandEvent(evt) => match evt {
                WaylandEvent::Output(evt, output) => match evt {
                    OutputEvent::Created(Some(info)) => {
                        if let Some((width, height)) = info.logical_size {
                            let id = self.next_surface_id();
                            self.layer_surfaces.insert(
                                id.clone(),
                                LayerSurface {
                                    output: output.clone(),
                                },
                            );
                            return get_layer_surface(SctkLayerSurfaceSettings {
                                id,
                                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                                namespace: "workspaces".into(),
                                layer: Layer::Overlay,
                                size: Some((Some(width as _), Some(height as _))),
                                output: IcedOutput::Output(output),
                                ..Default::default()
                            });
                        }
                    }
                    OutputEvent::Removed => {
                        if let Some((id, _)) = self
                            .layer_surfaces
                            .iter()
                            .find(|(_id, surface)| &surface.output == &output)
                        {
                            let id = *id;
                            self.layer_surfaces.remove(&id).unwrap();
                        }
                    }
                    // TODO handle update/remove
                    _ => {}
                },
                _ => {}
            },
            Msg::Wayland(evt) => {
                println!("{:?}", evt);
                match evt {
                    wayland::Event::Workspaces(workspaces) => {
                        // XXX efficiency
                        // XXX removal
                        self.workspaces = Vec::new();
                        for (output, workspace) in workspaces {
                            self.workspaces.push(Workspace {
                                name: workspace.name,
                                handle: workspace.handle,
                                output,
                                img: None,
                            });
                            println!("add workspace");
                        }
                    }
                    wayland::Event::WorkspaceCapture(workspace, image) => {
                        // XXX performance
                        for i in &mut self.workspaces {
                            if &i.handle == &workspace {
                                i.img = Some(image.clone());
                            }
                        }
                    }
                }
            }
            Msg::Close => {
                std::process::exit(0);
            }
            Msg::Closed(_) => {}
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        let events = iced::subscription::events_with(|evt, _| {
            //println!("{:?}", evt);
            if let iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(evt)) = evt
            {
                Some(Msg::WaylandEvent(evt))
            } else if let iced::Event::Keyboard(iced::keyboard::Event::KeyReleased {
                key_code: KeyCode::Escape,
                modifiers: _,
            }) = evt
            {
                Some(Msg::Close)
            } else {
                None
            }
        });
        iced::Subscription::batch(vec![events, wayland::subscription().map(Msg::Wayland)])
    }

    fn view(&self, id: SurfaceIdWrapper) -> cosmic::Element<Msg> {
        use iced::widget::*;
        if let SurfaceIdWrapper::LayerSurface(id) = id {
            if let Some(surface) = self.layer_surfaces.get(&id) {
                return layer_surface(self, surface);
            }
        };
        text("workspaces").into()
    }

    fn close_requested(&self, id: SurfaceIdWrapper) -> Msg {
        Msg::Closed(id)
    }
}

fn layer_surface<'a>(app: &'a App, surface: &'a LayerSurface) -> cosmic::Element<'a, Msg> {
    //workspaces_sidebar(app.workspaces.iter().filter(|i| &i.output == &surface.output))
    workspaces_sidebar(app.workspaces.iter())
}

fn workspace_sidebar_entry(workspace: &Workspace) -> cosmic::Element<Msg> {
    // x to close
    // captured preview
    // number      name
    // - selectable
    iced::widget::column![
        iced::widget::Image::new(
            workspace
                .img
                .clone()
                .unwrap_or_else(|| iced::widget::image::Handle::from_pixels(
                    0,
                    0,
                    vec![0, 0, 0, 255]
                ))
        ),
        iced::widget::text(&workspace.name)
    ]
    .height(iced::Length::Fill)
    .width(iced::Length::Fill)
    .into()
}

fn workspaces_sidebar<'a>(
    workspaces: impl Iterator<Item = &'a Workspace>,
) -> cosmic::Element<'a, Msg> {
    iced::widget::column(workspaces.map(workspace_sidebar_entry).collect()).into()
    // New workspace
}

/*
fn window_preview(&Window) -> cosmic::Element<Msg> {
   // capture of window
   // - selectable
   // name of window
}

fn window_previews(windows: &[Window]) -> cosmic::Element<Msg> {
    iced::widgets::row(windows.iter().map(window_preview).collect())
}
*/

pub fn main() -> iced::Result {
    App::run(iced::Settings {
        antialiasing: true,
        exit_on_close_request: false,
        initial_surface: InitialSurface::LayerSurface(SctkLayerSurfaceSettings {
            keyboard_interactivity: KeyboardInteractivity::None,
            namespace: "ignore".into(),
            size: Some((Some(1), Some(1))),
            layer: Layer::Background,
            ..Default::default()
        }),
        ..iced::Settings::default()
    })
}
