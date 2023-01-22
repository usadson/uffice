// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::time::Duration;

use windows::Win32::System::Com::CoInitialize;
use winit::event::ElementState;
use winit::event::VirtualKeyCode;
use winit::window::Window;
use winit::{
    event::{
        DeviceEvent,
        MouseScrollDelta,
    },
    event_loop::EventLoopProxy,
};

use crate::gui::Brush;
use crate::gui::Position;
use crate::gui::Rect;
use crate::gui::painter::PaintQuality;
use crate::gui::{
    AppEvent,
    Color,
    Size,

    animate::{
        Animated,
        Zoomer,
    },
    painter::{
        Painter,
        PainterCache,
    },
    scroll::Scroller,
    view::{
        View,
        document_view::VERTICAL_PAGE_MARGIN
    },
};
use crate::user_settings::SettingChangeNotification;
use crate::user_settings::SettingChangeOrigin;
use crate::user_settings::SettingChangeSubscriber;
use crate::user_settings::SettingName;
use crate::user_settings::UserSettings;

/// The background color of the application. This is the color under the pages.
const APPLICATION_BACKGROUND_COLOR: Color = Color::from_rgb(29, 28, 33);

/// After how much time should a tooltip be shown (if applicable).
///
/// The following is used as a recommendation:
///     https://ux.stackexchange.com/a/360
const TOOLTIP_TIMEOUT: Duration = Duration::from_millis(500);

const TOOLTIP_BACKGROUND_COLOR: Color = Color::from_rgb(211, 211, 211);
const TOOLTIP_BORDER_COLOR: Color = Color::from_rgb(168, 168, 168);

pub fn load_archive_file_to_string(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Option<Rc<String>> {
    match archive.by_name(name) {
        Ok(zip_document) => Some(Rc::new(std::io::read_to_string(zip_document)
                .expect("Failed to read"))),
        Err(e) => {
            println!("Error: {} for name \"{}\"", e, name);
            None
        }
    }
}

enum TooltipState {
    /// The mouse was moved but the timeout didn't expire yet.
    Unchecked,

    /// The tooltip is visible.
    Visible,

    /// The mouse hasn't moved after the timeout but there is no text to
    /// display.
    NotApplicable,
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TabId(usize);

unsafe impl Sync for TabId {}
unsafe impl Send for TabId {}

#[derive(Debug, PartialEq, Eq)]
pub enum TabState {
    Loading,
    Ready,
}

enum TabEvent {
    Layout {
        painter: Arc<RefCell<dyn Painter>>,
    },
    Paint {
        painter: Arc<RefCell<dyn Painter>>,
        window_size: Size<u32>,

        start_y: f32,
        zoom: f32,
    },
}

unsafe impl Send for TabEvent {}

pub struct TabFinishPaintInfo {
    content_height: f32
}

pub struct Tab {
    state: TabState,

    #[allow(dead_code)] // this will be used in the future for saving
    path: PathBuf,

    scroller: Scroller,
    zoomer: Zoomer,

    tab_event_sender: Sender<TabEvent>,

    /// Sent when the event was finished.
    finished_paint_receiver: Receiver<TabFinishPaintInfo>,
}

impl Tab {
    pub fn new(id: TabId, path: PathBuf, event_loop_proxy: EventLoopProxy<AppEvent>) -> Self {
        let (proxy_tx, proxy_rx) = channel();
        let (tab_event_sender, tab_event_receiver) = channel();
        let (finished_paint_sender, finished_paint_receiver) = channel();

        let path_str = path.to_str().unwrap().to_owned();
        std::thread::spawn(move || {
            let proxy: EventLoopProxy<AppEvent> = proxy_rx.recv().unwrap();
            drop(proxy_rx);

            let mut view = None;
            proxy.send_event(AppEvent::PainterRequest).unwrap();

            unsafe {
                CoInitialize(None).expect("Failed to initialize COM, this is needed because this is another thread. Maybe we could look into MTA using roapi?");
            }

            for event in tab_event_receiver {
                match event {
                    TabEvent::Layout { painter } => {
                        if view.is_some() {
                            continue;
                        }

                        let text_calculator = {
                            let painter = &mut *painter.as_ref().borrow_mut();
                            painter.text_calculator()
                        };
                        assert!(painter.try_borrow_mut().is_ok(), "Borrow painter as mutable failed after getting text calculator?");
                        finished_paint_sender.send(TabFinishPaintInfo { content_height: 0.0 }).unwrap();

                        let mut text_calculator = text_calculator.as_ref().borrow_mut();
                        view = Some(View::Document(crate::gui::view::document_view::DocumentView::new(&path_str, &mut *text_calculator)));

                        assert!(painter.try_borrow_mut().is_ok(), "Borrow painter as mutable failed after getting text calculator?");
                        proxy.send_event(AppEvent::TabBecameReady(id)).unwrap();
                    }
                    TabEvent::Paint{ painter, window_size, start_y, zoom } => {
                        let mut content_height = 0.0;

                        // Scope this so the painter borrow is dropped before
                        // sending the finish message.
                        if let Some(view) = &mut view {
                            let painter = &mut *painter.as_ref().borrow_mut();
                            view.handle_event(&mut crate::gui::view::Event::Paint(crate::gui::view::PaintEvent {
                                opaqueness: 1.0,
                                painter,
                                start_y,
                                window_size,
                                zoom
                            }));

                            proxy.send_event(AppEvent::TabPainted{
                                tab_id: id,
                                total_content_height: view.calculate_content_height()
                            }).unwrap();

                            content_height = view.calculate_content_height();
                        }

                        assert!(painter.try_borrow_mut().is_ok(), "Borrow painter as mutable failed after finish paint?");
                        finished_paint_sender.send(TabFinishPaintInfo{
                            content_height
                        }).unwrap();
                    }
                }
            }
        });

        proxy_tx.send(event_loop_proxy.clone()).unwrap();
        drop(proxy_tx);

        Self {
            state: TabState::Loading,
            path,
            scroller: Scroller::new(),
            zoomer: Zoomer::new(),
            tab_event_sender,
            finished_paint_receiver,
        }
    }

    pub fn on_became_ready(&mut self) {
        self.state = TabState::Ready;
    }

    pub fn on_tab_painted(&mut self, total_content_height: f32) {
        self.scroller.content_height = total_content_height;
    }

    fn on_paint(&mut self, event: &crate::gui::app::PaintEvent) {
        if self.state == TabState::Loading {
            return;
        }

        assert!(event.painter.try_borrow_mut().is_ok(), "Failed to painter borrow as mutable; we can never send the PaintEvent to the tab!");

        let size = event.window.inner_size().to_logical(event.window.scale_factor());
        let zoom_level = self.zoomer.zoom_factor();
        self.tab_event_sender.send(TabEvent::Paint {
            painter: event.painter.clone(),
            window_size: Size::new(size.width, size.height),
            start_y: (VERTICAL_PAGE_MARGIN - self.scroller.content_height * self.scroller.position()) * zoom_level,
            zoom: zoom_level
        }).unwrap();

        self.scroller.content_height = self.finished_paint_receiver.recv().unwrap().content_height;
        assert!(event.painter.try_borrow_mut().is_ok(), "Failed to painter borrow as mutable while finish_paint was received!");
    }

    /// Returns whether or not to repaint.
    pub fn on_scroll(&mut self, delta: MouseScrollDelta, keyboard: &uffice_lib::Keyboard) -> bool {
        if let MouseScrollDelta::LineDelta(_left, top) = delta {
            if keyboard.is_control_key_dow() {
                if top > 0.2 {
                    return self.zoomer.increase_zoom_level();
                }

                if top < -0.2 {
                    return self.zoomer.decrease_zoom_level();
                }

                return false;
            }

            self.scroller.scroll_lines(top);
            return true;
        }

        return false;
    }

    pub fn has_running_animations(&mut self) -> bool {
        self.zoomer.has_running_animation() || self.scroller.has_running_animation()
    }
}

impl SettingChangeSubscriber for Tab {
    fn setting_changed(&mut self, notification: &SettingChangeNotification) {
        self.scroller.setting_changed(notification);
        self.zoomer.setting_changed(notification);
    }

    fn settings_loaded(&mut self, settings: &UserSettings) {
        self.scroller.settings_loaded(settings);
        self.zoomer.settings_loaded(settings);
    }
}

pub struct App {
    event_loop_proxy: EventLoopProxy<AppEvent>,

    next_tab_id: usize,
    current_visible_tab: Option<TabId>,
    tabs: HashMap<TabId, Tab>,

    keyboard: uffice_lib::Keyboard,
    user_settings: UserSettings,

    previous_frame_had_running_animations: bool,
}

impl App {
    pub fn new(window: &mut winit::window::Window, event_loop_proxy: EventLoopProxy<AppEvent>, first_file_to_open: String) -> Self {
        window.set_title(&format!("{} - {}", crate::gui::app::formatted_base_title(), first_file_to_open));

        let mut app = Self {
            event_loop_proxy,
            next_tab_id: 1000,
            current_visible_tab: None,
            tabs: HashMap::new(),

            keyboard: uffice_lib::Keyboard::new(),
            user_settings: UserSettings::load(),

            previous_frame_had_running_animations: false,
        };

        app.add_tab(PathBuf::from(first_file_to_open));

        app
    }

    fn add_tab(&mut self, path: PathBuf) {
        let tab_id = TabId(self.next_tab_id);
        self.next_tab_id += 1;

        let mut tab = Tab::new(tab_id, path, self.event_loop_proxy.clone());
        tab.settings_loaded(&self.user_settings);
        self.tabs.insert(tab_id, tab);

        if self.current_visible_tab.is_some() {
            return;
        }

        self.current_visible_tab = Some(tab_id);
    }

    fn handle_user_event(&mut self, window: &mut winit::window::Window, event: AppEvent) {
        match event {
            AppEvent::TabBecameReady(tab_id) => {
                self.tabs.get_mut(&tab_id).unwrap().on_became_ready();

                if Some(tab_id) == self.current_visible_tab {
                    window.request_redraw();
                }
            }

            AppEvent::TabPainted { tab_id, total_content_height } => {
                self.tabs.get_mut(&tab_id).unwrap().on_tab_painted(total_content_height);
            }

            AppEvent::PainterRequest => ()
        }
    }

    /// Called when the specified key is pressed (for the first time, not held).
    pub fn on_key_pressed(&mut self, key: VirtualKeyCode, window: &mut Window) {
        println!("Key: {:?}", key);
        match key {
            VirtualKeyCode::Minus => {
                if self.keyboard.is_control_key_dow() {
                    if let Some(current_tab_id) = self.current_visible_tab {
                        if self.tabs.get_mut(&current_tab_id).unwrap().zoomer.decrease_zoom_level() {
                            window.request_redraw();
                        }
                    }
                }
            }

            VirtualKeyCode::Equals => {
                if self.keyboard.is_control_key_dow() {
                    if let Some(current_tab_id) = self.current_visible_tab {
                        if self.tabs.get_mut(&current_tab_id).unwrap().zoomer.increase_zoom_level() {
                            window.request_redraw();
                        }
                    }
                }
            }

            VirtualKeyCode::Key0 => {
                self.broadcast_setting_changed(SettingChangeOrigin::System, SettingName::EnableAnimations);
            }

            _ => ()
        }
    }

    fn broadcast_setting_changed(&mut self, origin: SettingChangeOrigin, setting_name: SettingName) {
        let notification = SettingChangeNotification {
            origin, setting_name, settings: &self.user_settings
        };
        for tab in self.tabs.values_mut() {
            tab.setting_changed(&notification);
        }
    }
}

impl crate::gui::app::GuiApp for App {

    fn on_event(&mut self, window: &mut winit::window::Window, event: winit::event::Event<AppEvent>) {
        use winit::event::Event;
        match event {

            // TODO: Receive system parameter change updates. This is necessary
            //       to provide a smooth user experience. Examples of such an
            //       event include the WM_SETTINGCHANGE of the Windows API.

            Event::DeviceEvent {
                event: DeviceEvent::MouseWheel { delta }, ..
            } => {
                if let Some(current_tab_id) = self.current_visible_tab {
                    let should_scroll = self.tabs.get_mut(&current_tab_id).unwrap().on_scroll(delta, &self.keyboard);
                    if should_scroll {
                        window.request_redraw();
                    }
                }
            }

            Event::DeviceEvent { event: DeviceEvent::Key(keyboard), .. } => {

                if let Some(key) = keyboard.virtual_keycode {
                    if keyboard.state == ElementState::Pressed && !self.keyboard.is_down(key) {
                        self.on_key_pressed(key, window);
                    }
                }

                self.keyboard.handle_input_event(&keyboard);
            }

            Event::UserEvent(app_event) => self.handle_user_event(window, app_event),

            _ => ()
        }
    }

    fn paint(&mut self, event: &mut crate::gui::app::PaintEvent) {
        let window_size = event.window.inner_size().to_logical::<f32>(event.window.scale_factor()).into();

        assert!(event.painter.try_borrow_mut().is_ok(), "Failed to painter borrow as mutable; cannot paint App");
        event.painter.as_ref().borrow_mut().paint_rect(Brush::SolidColor(APPLICATION_BACKGROUND_COLOR),
            Rect::from_position_and_size(Position::new(0.0, 0.0), window_size));

        if let Some(current_tab_id) = self.current_visible_tab {
            let current_tab = self.tabs.get_mut(&current_tab_id).unwrap();

            let has_animations = current_tab.has_running_animations();
            let quality = if has_animations {
                PaintQuality::AvoidResourceRescalingForDetail
            } else {
                PaintQuality::Full
            };
            event.painter.as_ref().borrow_mut().switch_cache(PainterCache::Document(current_tab_id.0), quality);
            current_tab.on_paint(&event);

            let mut painter = event.painter.as_ref().borrow_mut();
            painter.switch_cache(PainterCache::UI, PaintQuality::Full);

            current_tab.scroller.paint(event.window, &mut *painter);

            if has_animations {
                event.should_redraw_again = true;
                self.previous_frame_had_running_animations = true;
            } else if self.previous_frame_had_running_animations {
                self.previous_frame_had_running_animations = false;
                event.should_redraw_again = true;
            }
        }
    }

    /// This function is called in response to a `AppEvent::PainterRequest`.
    fn receive_painter(&mut self, painter: Arc<RefCell<dyn Painter>>) {
        for tab in self.tabs.values() {
            if tab.state == TabState::Loading {
                tab.tab_event_sender.send(TabEvent::Layout { painter: painter.clone() }).unwrap();
                tab.finished_paint_receiver.recv().unwrap();
            }
        }
    }

}
