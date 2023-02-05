// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::time::Duration;

use windows::Win32::System::Com::CoInitialize;
use windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR;
use windows::Win32::UI::WindowsAndMessaging::MB_OK;
use windows::Win32::UI::WindowsAndMessaging::MessageBoxA;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;
use winit::window::Window;
use winit::{
    event::{
        DeviceEvent,
        MouseScrollDelta,
    },
    event_loop::EventLoopProxy,
};

use crate::gui::Brush;
use crate::gui::EventVisualReaction;
use crate::gui::InteractionState;
use crate::gui::MouseMoveEvent;
use crate::gui::Position;
use crate::gui::Rect;
use crate::gui::painter::FontSpecification;
use crate::gui::painter::FontWeight;
use crate::gui::painter::PaintQuality;
use crate::gui::widget::TabWidget;
use crate::gui::widget::TabWidgetItem;
use crate::gui::widget::Widget;
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

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TabId(usize);

impl Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

unsafe impl Sync for TabId {}
unsafe impl Send for TabId {}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TabState {
    Loading,
    Ready,
    Crashed,
    Finished,
}

enum TabEvent {
    Layout {
        painter: Arc<RefCell<dyn Painter>>,
    },
    Paint {
        painter: Arc<RefCell<dyn Painter>>,
        content_rect: Rect<f32>,

        start_y: f32,
        zoom: f32,
    },
}

unsafe impl Send for TabEvent {}

pub struct TabFinishPaintInfo {
    content_height: f32
}

#[derive(Debug, PartialEq, Eq)]
pub enum TabCrashKind {
    Win32ComFailure(String)
}

unsafe impl Send for TabCrashKind {}

#[derive(Debug, PartialEq, Eq)]
pub struct TabCrashReason {
    pub origin: &'static str,
    pub description: &'static str,
    pub kind: TabCrashKind,
}

unsafe impl Send for TabCrashReason {}

pub struct Tab {
    state: TabState,
    join_handle: Option<std::thread::JoinHandle<Result<(), TabCrashReason>>>,
    crash_reason: Option<TabCrashReason>,

    #[allow(dead_code)] // this will be used in the future for saving
    path: PathBuf,

    scroller: Scroller,
    zoomer: Zoomer,

    tab_event_sender: Sender<TabEvent>,

    /// Sent when the event was finished.
    finished_paint_receiver: Receiver<TabFinishPaintInfo>,

    /// How much of the document is loaded, between 0.0 and 1.0.
    loading_progress: f32,
    /// How many pages were painted.
    page_count: usize,
}

impl Tab {
    pub fn new(id: TabId, path: PathBuf, event_loop_proxy: EventLoopProxy<AppEvent>) -> Self {
        let (proxy_tx, proxy_rx) = channel();
        let (tab_event_sender, tab_event_receiver) = channel();
        let (finished_paint_sender, finished_paint_receiver) = channel();

        let path_str = path.to_str().unwrap().to_owned();
        let join_handle = std::thread::Builder::new()
                .name(format!("Tab Manager #{}", id))
                .spawn(move || -> Result<(), TabCrashReason> {
            let proxy: EventLoopProxy<AppEvent> = proxy_rx.recv().unwrap();
            drop(proxy_rx);

            let mut view = None;
            proxy.send_event(AppEvent::PainterRequest).unwrap();

            #[cfg(windows)]
            unsafe {
                if let Err(err) = CoInitialize(None) {
                    _ = proxy.send_event(AppEvent::TabCrashed { tab_id: id });
                    return Err(TabCrashReason{
                        origin: "CoInitialize",
                        description: "Failed to initialize COM, this is needed because this is another thread. Maybe we could look into MTA using roapi?",
                        kind: TabCrashKind::Win32ComFailure(err.to_string())
                    });
                }
            }

            for event in tab_event_receiver {
                match event {
                    TabEvent::Layout { painter } => {
                        // if view.is_some() {
                        //     continue;
                        // }

                        let text_calculator = {
                            let painter = &mut *painter.as_ref().borrow_mut();
                            painter.text_calculator()
                        };
                        assert!(painter.try_borrow_mut().is_ok(), "Borrow painter as mutable failed after getting text calculator?");
                        finished_paint_sender.send(TabFinishPaintInfo { content_height: 0.0 }).unwrap();

                        let mut text_calculator = text_calculator.as_ref().borrow_mut();
                        view = Some(View::Document(crate::gui::view::document_view::DocumentView::new(&path_str, &mut *text_calculator,
                            &|progress| {
                                _ = proxy.send_event(AppEvent::TabProgressed { tab_id: id, progress });
                            }
                        )));

                        proxy.send_event(AppEvent::TabBecameReady(id)).unwrap();
                    }
                    TabEvent::Paint{ painter, content_rect, start_y, zoom } => {
                        let mut content_height = 0.0;

                        // Scope this so the painter borrow is dropped before
                        // sending the finish message.
                        if let Some(view) = &mut view {
                            let painter = &mut *painter.as_ref().borrow_mut();
                            view.handle_event(&mut crate::gui::view::Event::Paint(crate::gui::view::PaintEvent {
                                content_rect,
                                opaqueness: 1.0,
                                painter,
                                start_y,
                                zoom
                            }));

                            proxy.send_event(AppEvent::TabPainted{
                                tab_id: id,
                                total_content_height: view.calculate_content_height(),
                                page_count: view.page_count().unwrap_or(0),
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

            Ok(())
        }).unwrap();

        proxy_tx.send(event_loop_proxy.clone()).unwrap();
        drop(proxy_tx);

        Self {
            state: TabState::Loading,
            join_handle: Some(join_handle),
            crash_reason: None,
            path,
            scroller: Scroller::new(),
            zoomer: Zoomer::new(),
            tab_event_sender,
            finished_paint_receiver,
            loading_progress: 0.0,
            page_count: 0,
        }
    }

    pub fn on_became_ready(&mut self) {
        self.state = TabState::Ready;
    }

    pub fn on_tab_painted(&mut self, total_content_height: f32, page_count: usize) {
        self.scroller.content_height = total_content_height;
        self.page_count = page_count;
    }

    pub fn on_tab_progressed(&mut self, progress: f32) {
        self.loading_progress = progress;
    }

    pub fn check_state(&mut self) -> TabState {
        if self.join_handle.is_some() {
            if self.state != TabState::Crashed && self.join_handle.as_ref().unwrap().is_finished(){
                let join_handle = self.join_handle.take().unwrap();

                self.state = match join_handle.join().unwrap() {
                    Ok(..) => TabState::Finished,
                    Err(err) => {
                        self.crash_reason = Some(err);
                        TabState::Crashed
                    }
                }
            }
        }

        self.state
    }

    /// Lets the user know that the tab is loading.
    fn paint_loading_screen(&mut self, event: &crate::gui::app::PaintEvent, content_rect: Rect<f32>) {
        let painter = &mut *event.painter.as_ref().borrow_mut();

        let text = format!("Loading... {:.1}%", self.loading_progress * 100.0);

        painter.select_font(FontSpecification::new("Segoe UI", 24.0, FontWeight::Regular))
            .expect("Failed to load UI font");

        let size = painter.paint_text(Brush::SolidColor(Color::TRANSPARENT), Position::new(0.0, 0.0), &text, None);
        let position = Position::new(
            content_rect.left() + (content_rect.width() - size.width()) / 2.0,
            content_rect.top() + (content_rect.height() - size.height()) / 2.0,
        );

        painter.paint_text(Brush::SolidColor(Color::BLACK), position, &text, None);
    }

    fn on_paint(&mut self, event: &crate::gui::app::PaintEvent, content_rect: Rect<f32>) {
        if self.state == TabState::Loading {
            self.paint_loading_screen(event, content_rect);
            return;
        }

        assert!(event.painter.try_borrow_mut().is_ok(), "Failed to painter borrow as mutable; we can never send the PaintEvent to the tab!");

        let zoom_level = self.zoomer.zoom_factor();
        self.tab_event_sender.send(TabEvent::Paint {
            painter: event.painter.clone(),
            content_rect,
            start_y: (VERTICAL_PAGE_MARGIN - self.scroller.content_height * self.scroller.position()) * zoom_level,
            zoom: zoom_level
        }).unwrap();

        self.scroller.content_height = self.finished_paint_receiver.recv().unwrap().content_height;
        assert!(event.painter.try_borrow_mut().is_ok(), "Failed to painter borrow as mutable while finish_paint was received!");
    }

    /// Returns whether or not to repaint.
    pub fn on_scroll(&mut self, delta: MouseScrollDelta, keyboard: &uffice_lib::Keyboard) -> bool {
        if let MouseScrollDelta::LineDelta(_left, top) = delta {
            if keyboard.is_control_key_down() {
                if top > 0.2 {
                    return self.zoomer.increase_zoom_level();
                }

                if top < -0.2 {
                    return self.zoomer.decrease_zoom_level();
                }

                return false;
            }

            return self.scroller.scroll_lines(top);
        }

        return false;
    }

    pub fn has_running_animations(&mut self) -> bool {
        self.zoomer.has_running_animation() || self.scroller.has_running_animation()
    }

    pub fn on_mouse_input(&mut self, mouse_position: Position<f32>, button: MouseButton, state: ElementState) {
        self.scroller.on_mouse_input(mouse_position, button, state);
    }

    pub fn on_mouse_move(&mut self, event: &mut MouseMoveEvent) {
        if self.scroller.bar_rect.is_inside_inclusive(event.position) || self.scroller.interaction_state != InteractionState::Default {
            self.scroller.on_mouse_move(event);
        }
    }

    pub fn on_window_focus_lost(&mut self) {
        self.scroller.on_window_focus_lost();
    }

    fn reload(&mut self) {
        assert!(self.state == TabState::Ready);
        self.state = TabState::Loading;
    }
}

impl TabWidgetItem for Tab {
    fn title(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
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
    tabs: BTreeMap<TabId, Tab>,
    tab_widget: TabWidget<Tab>,

    keyboard: uffice_lib::Keyboard,
    mouse_position: Position<f32>,
    mouse_inside_window: bool,
    user_settings: UserSettings,

    previous_frame_had_running_animations: bool,
}

impl App {
    pub fn new(window: &mut winit::window::Window, event_loop_proxy: EventLoopProxy<AppEvent>, files_to_open: Vec<String>) -> Self {
        let mut app = Self {
            event_loop_proxy,
            next_tab_id: 1000,
            current_visible_tab: None,
            tabs: Default::default(),
            tab_widget: TabWidget::new(),

            keyboard: uffice_lib::Keyboard::new(),
            mouse_position: Position::new(0.0, 0.0),
            mouse_inside_window: false,
            user_settings: UserSettings::load(),

            previous_frame_had_running_animations: false,
        };

        for file in files_to_open {
            app.add_tab(file.into(), window);
        }

        app
    }

    fn add_tab(&mut self, path: PathBuf, window: &mut winit::window::Window) -> TabId {
        let path = path.canonicalize().unwrap_or(path);
        let tab_id = TabId(self.next_tab_id);
        self.next_tab_id += 1;

        let mut tab = Tab::new(tab_id, path, self.event_loop_proxy.clone());
        tab.settings_loaded(&self.user_settings);
        self.tabs.insert(tab_id, tab);

        self.save_restore_point();

        if self.current_visible_tab.is_none() {
            self.switch_to_tab(tab_id, window);
        }

        tab_id
    }

    /// Check the digit key (1 - 9).
    fn check_key_digit(&mut self, digit: u8, window: &mut winit::window::Window) {
        if !self.keyboard.is_control_key_down() {
            return;
        }

        if let Some(tab_id) = self.tabs.keys().nth((digit - 1) as _) {
            // Is the tab already the current visible tab?
            if Some(*tab_id) == self.current_visible_tab {
                return;
            }

            self.current_visible_tab = Some(*tab_id);
            window.request_redraw();
        }
    }

    /// Close the current visible tab.
    fn close_current_tab(&mut self) {
        let Some(tab_id) = self.current_visible_tab else {
            return;
        };

        self.tabs.remove(&tab_id);
        self.current_visible_tab = self.tabs.keys().next().copied();
    }

    fn switch_to_tab(&mut self, tab_id: TabId, window: &mut winit::window::Window) {
        window.set_title(&format!("{} - {}", crate::gui::app::formatted_base_title(), self.tabs.get(&tab_id).unwrap().path.display()));

        self.current_visible_tab = Some(tab_id);
        window.request_redraw();
    }

    fn handle_tab_mouse_move(&mut self, event: &mut MouseMoveEvent) {
        let Some(tab_id) = self.current_visible_tab else {
            return;
        };

        let Some(tab) = self.tabs.get_mut(&tab_id) else {
            return;
        };

        tab.on_mouse_move(event);
    }

    fn handle_user_event(&mut self, window: &mut winit::window::Window, event: AppEvent) {
        match event {
            AppEvent::TabBecameReady(tab_id) => {
                let Some(tab) = self.tabs.get_mut(&tab_id) else {
                    println!("[App] Warning: TabBecameReady: Tab not found/closed.");
                    return;
                };

                tab.on_became_ready();

                if Some(tab_id) == self.current_visible_tab {
                    window.request_redraw();
                }
            }

            AppEvent::TabPainted { tab_id, total_content_height, page_count } => {
                if let Some(tab) = self.tabs.get_mut(&tab_id) {
                    tab.on_tab_painted(total_content_height, page_count);
                } else {
                    println!("[App] Warning: TabPainted: Tab not found/closed.");
                }
            }

            AppEvent::TabProgressed { tab_id, progress } => {
                if let Some(tab) = self.tabs.get_mut(&tab_id) {
                    tab.on_tab_progressed(progress);
                    window.request_redraw();
                } else {
                    println!("[App] Warning: TabProgressed: Tab not found/closed.");
                }
            }

            AppEvent::TabCrashed { tab_id } => {
                let tab = self.tabs.remove(&tab_id);
                if tab.is_none() {
                    return;
                }
                let tab = tab.unwrap();

                if let Some(current_tab) = self.current_visible_tab {
                    if current_tab == tab_id {
                        if let Some(first) = self.tabs.keys().next() {
                            self.current_visible_tab = Some(*first);
                        } else {
                            self.current_visible_tab = None;
                        }
                    }
                }

                unsafe {
                    let message = format!("ID: {}\r\nReason: {:?}", tab_id, tab.crash_reason);
                    MessageBoxA(None, windows::core::PCSTR(message.as_ptr()), windows::core::PCSTR("Tab Crashed".as_ptr()), MB_ICONERROR | MB_OK);
                }
            }

            AppEvent::PainterRequest => ()
        }
    }

    /// Called when the specified key is pressed (for the first time, not held).
    pub fn on_key_pressed(&mut self, key: VirtualKeyCode, window: &mut Window) {
        match key {
            VirtualKeyCode::W => {
                if self.keyboard.is_control_key_down() {
                    self.close_current_tab();
                    window.request_redraw();
                }
            }

            VirtualKeyCode::Minus => {
                if self.keyboard.is_control_key_down() {
                    if let Some(current_tab_id) = self.current_visible_tab {
                        if self.tabs.get_mut(&current_tab_id).unwrap().zoomer.decrease_zoom_level() {
                            window.request_redraw();
                        }
                    }
                }
            }

            VirtualKeyCode::Equals => {
                if self.keyboard.is_control_key_down() {
                    if let Some(current_tab_id) = self.current_visible_tab {
                        if self.tabs.get_mut(&current_tab_id).unwrap().zoomer.increase_zoom_level() {
                            window.request_redraw();
                        }
                    }
                }
            }

            VirtualKeyCode::Key1 => self.check_key_digit(1, window),
            VirtualKeyCode::Key2 => self.check_key_digit(2, window),
            VirtualKeyCode::Key3 => self.check_key_digit(3, window),
            VirtualKeyCode::Key4 => self.check_key_digit(4, window),
            VirtualKeyCode::Key5 => self.check_key_digit(5, window),
            VirtualKeyCode::Key6 => self.check_key_digit(6, window),
            VirtualKeyCode::Key7 => self.check_key_digit(7, window),
            VirtualKeyCode::Key8 => self.check_key_digit(8, window),
            VirtualKeyCode::Key9 => self.check_key_digit(9, window),

            VirtualKeyCode::F5 => {
                if let Some(tab_id) = self.current_visible_tab {
                    let tab = self.tabs.get_mut(&tab_id).unwrap();
                    tab.reload();
                    _ = self.event_loop_proxy.send_event(AppEvent::PainterRequest);
                }
            }

            #[cfg(debug_assertions)]
            VirtualKeyCode::F9 => window.request_redraw(),

            VirtualKeyCode::F10 => {
                if let Some(current_tab_id) = self.current_visible_tab {
                    let current_tab = self.tabs.get(&current_tab_id).unwrap();
                    crate::platform::open_file_user(current_tab.path.to_str().unwrap());
                }
            }

            #[cfg(debug_assertions)]
            VirtualKeyCode::Pause => {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                }
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

    /// Saves the current state in case that the application crashes or the
    /// system is rebooted automatically.
    fn save_restore_point(&mut self) {
        crate::platform::save_restore_arguments(crate::CommandLineArguments{
            files: self.tabs.values().map(|tab| tab.path.to_str().unwrap().to_owned()).collect(),

            ..Default::default()
        })
    }

    fn selected_tab_to_index(&self) -> Option<usize> {
        let Some(tab_id) = self.current_visible_tab else {
            return None
        };

        self.tabs.keys().position(|id| *id == tab_id)
    }

    fn paint_status_bar(&self, mut painter: RefMut<dyn Painter>, window_size: Size<f32>) {
        let Some(tab_id) = self.current_visible_tab else {
            return;
        };

        let Some(tab) = self.tabs.get(&tab_id) else {
            return
        };

        let size = Size::new(window_size.width(), 15.0);
        let padding = 3.3;

        let position = Position::new(0.0, window_size.height() - size.height());
        painter.paint_rect(Brush::SolidColor(Color::from_rgb(0x22, 0x22, 0x22)),
                Rect::from_position_and_size(position, size));

        let text = format!("1238 words,  {} pages,   {}% zoom", tab.page_count, tab.zoomer.zoom_factor_unanimated() * 100.0);

        painter.select_font(FontSpecification::new("Segoe UI", 8.0, FontWeight::Regular)).unwrap();
        painter.paint_text(Brush::SolidColor(Color::from_rgb(0xCC, 0xCC, 0xCC)), Position::new(padding, position.y()), &text, None);

        drop(painter);
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
                if !self.mouse_inside_window {
                    return;
                }

                if let Some(current_tab_id) = self.current_visible_tab {
                    let should_scroll = self.tabs.get_mut(&current_tab_id).unwrap().on_scroll(delta, &self.keyboard);
                    if should_scroll {
                        window.request_redraw();
                    }
                }
            }

            Event::WindowEvent { event: WindowEvent::CursorLeft { .. }, .. } => {
                self.mouse_inside_window = false;
            }

            Event::WindowEvent { event: WindowEvent::CursorEntered { .. }, .. } => {
                self.mouse_inside_window = false;
            }

            Event::WindowEvent { event: WindowEvent::DroppedFile(path), .. } => {
                let new_tab = self.add_tab(path, window);
                self.current_visible_tab = Some(new_tab);
                window.request_redraw();
            }

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                let size = size.to_logical(window.scale_factor());
                let size = Size::new(size.width, size.height);
                self.tab_widget.on_window_resize(size);
            }

            Event::WindowEvent { event: WindowEvent::Focused(is_focused), .. } => {
                if !is_focused {
                    self.tab_widget.on_window_focus_lost();
                    if let Some(tab_id) = self.current_visible_tab {
                        self.tabs.get_mut(&tab_id).unwrap().on_window_focus_lost();
                    }
                }
            }

            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                self.mouse_inside_window = true;

                let position = position.to_logical::<f32>(window.scale_factor());
                let position = Position::new(position.x, position.y);

                let mut event = MouseMoveEvent {
                    reaction: EventVisualReaction::Ignored,

                    position,
                    previous_position: self.mouse_position,
                    delta_x: position.x() - self.mouse_position.x(),
                    delta_y: position.y() - self.mouse_position.y(),
                };

                self.mouse_position = position;

                let was_inside_widget = self.tab_widget.rect().is_inside_inclusive(self.mouse_position);
                let is_inside_widget = self.tab_widget.rect().is_inside_inclusive(self.mouse_position);

                if was_inside_widget && !is_inside_widget {
                    self.tab_widget.on_mouse_leave(&mut event);
                    self.tab_widget.on_mouse_move(&mut event);
                } else if is_inside_widget && !was_inside_widget {
                    self.tab_widget.on_mouse_enter(&mut event);
                    self.tab_widget.on_mouse_move(&mut event);
                } else if is_inside_widget && was_inside_widget {
                    self.tab_widget.on_mouse_move(&mut event);
                }

                self.handle_tab_mouse_move(&mut event);

                if event.reaction == EventVisualReaction::ContentUpdated {
                    window.request_redraw();
                }
            }

            Event::WindowEvent { event: WindowEvent::MouseInput { button, state, .. }, .. } => {
                if self.tab_widget.rect().is_inside_inclusive(self.mouse_position) {
                    self.tab_widget.on_mouse_input(self.mouse_position, button, state);
                    return;
                }

                if let Some(tab_id) = self.current_visible_tab {
                    let tab = self.tabs.get_mut(&tab_id).unwrap();
                    tab.on_mouse_input(self.mouse_position, button, state);
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
        // event.painter.as_ref().borrow_mut().paint_rect(Brush::SolidColor(APPLICATION_BACKGROUND_COLOR),
        //     Rect::from_position_and_size(Position::new(0.0, 0.0), window_size));

        event.painter.as_ref().borrow_mut().paint_rect(Brush::Test,
            Rect::from_position_and_size(Position::new(0.0, 0.0), window_size));

        if let Some(current_tab_id) = self.current_visible_tab {
            let current_tab = self.tabs.get_mut(&current_tab_id).unwrap();

            let has_animations_at_beginning_of_paint = current_tab.has_running_animations();
            let quality = if has_animations_at_beginning_of_paint {
                PaintQuality::AvoidResourceRescalingForDetail
            } else {
                PaintQuality::Full
            };
            event.painter.as_ref().borrow_mut().switch_cache(PainterCache::Document(current_tab_id.0), quality);

            let content_rect = Rect::from_position_and_size(
                Position::new(0.0, self.tab_widget.rect().bottom),
                Size::new(
                    window_size.width() - current_tab.scroller.bar_rect.width(),
                    window_size.height() - self.tab_widget.rect().height() - 15.0
                )
            );
            current_tab.on_paint(&event, content_rect);

            let mut painter = event.painter.as_ref().borrow_mut();
            painter.switch_cache(PainterCache::UI, PaintQuality::Full);

            current_tab.scroller.paint(&mut *painter, content_rect);

            if has_animations_at_beginning_of_paint || current_tab.has_running_animations() {
                event.should_redraw_again = true;
                self.previous_frame_had_running_animations = true;
            } else if self.previous_frame_had_running_animations {
                self.previous_frame_had_running_animations = false;
                event.should_redraw_again = true;
            }
        }

        let mut painter = event.painter.borrow_mut();
        self.tab_widget.paint(&mut *painter, self.tabs.values(), self.selected_tab_to_index());
        self.paint_status_bar(painter, window_size);
    }

    /// This function is called in response to a `AppEvent::PainterRequest`.
    fn receive_painter(&mut self, painter: Arc<RefCell<dyn Painter>>) {
        for tab in self.tabs.values_mut() {
            if tab.check_state() == TabState::Loading {
                assert!(tab.finished_paint_receiver.try_recv().is_err());
                tab.tab_event_sender.send(TabEvent::Layout { painter: painter.clone() }).unwrap();
                tab.finished_paint_receiver.recv().unwrap();
            }
        }
    }

}
