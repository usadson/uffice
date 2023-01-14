// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::time::Instant;

use font_kit::family_name::FamilyName;

use sfml::SfBox;
use sfml::graphics::*;
use sfml::system::Vector2f;
use sfml::window::*;

use notify::{Watcher, RecursiveMode};
use windows::Win32::System::Com::CoInitialize;
use winit::event::DeviceEvent;
use winit::event::MouseScrollDelta;
use winit::event_loop::EventLoopProxy;

use crate::gui::AppEvent;
use crate::gui::Size;
use crate::gui::animate::Animator;
use crate::gui::animate::Zoomer;
use crate::gui::painter::Painter;
use crate::gui::painter::PainterCache;
use crate::gui::painter::TextCalculator;
use crate::gui::scroll::Scroller;
use crate::gui::view::View;
use crate::gui::view::document_view::VERTICAL_PAGE_MARGIN;
use crate::text_settings::Position;
use crate::wp;
use crate::wp::MouseEvent;

/// The background color of the application. This is the color under the pages.
const APPLICATION_BACKGROUND_COLOR: Color = Color::rgb(29, 28, 33);

/// After how much time should a tooltip be shown (if applicable).
///
/// The following is used as a recommendation:
///     https://ux.stackexchange.com/a/360
const TOOLTIP_TIMEOUT: Duration = Duration::from_millis(500);

const TOOLTIP_BACKGROUND_COLOR: Color = Color::rgb(211, 211, 211);
const TOOLTIP_BORDER_COLOR: Color = Color::rgb(168, 168, 168);

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

pub struct Application<'a> {
    archive_path: String,

    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,

    window: RenderWindow,
    cursor: SfBox<Cursor>,
    keyboard: uffice_lib::Keyboard,

    // In the future, we can make this a vector to have multiple tabs!
    view: Option<crate::gui::view::View>,

    interface_font: SfBox<Font>,

    is_draw_invalidated: Arc<AtomicBool>,
    scroller: Scroller,

    /// This defines how zoomed in or out the pages are.
    zoomer: Zoomer,

    mouse_position: Vector2f,
    last_mouse_move: Instant,
    tooltip_state: TooltipState,
    tooltip_text: String,

    rectangle_shape: RectangleShape<'a>,
}

fn load_interface_font() -> Option<SfBox<Font>> {
    let font_source = font_kit::source::SystemSource::new();
    let font_handle = font_source.select_best_match(&[
        FamilyName::Title(String::from("Segoe UI")),
        FamilyName::Title(String::from("Noto Sans")),
        FamilyName::SansSerif
    ], &font_kit::properties::Properties::new())
        .expect("Failed to find a system font!");

    match &font_handle {
        font_kit::handle::Handle::Memory { bytes, font_index: _ } => unsafe {
            Font::from_memory(&bytes.as_ref())
        }
        font_kit::handle::Handle::Path { path, font_index: _ } => {
            Font::from_file(path.to_str().unwrap())
        }
    }
}

impl<'a> Application<'a> {
    pub fn new(archive_path: String) -> Self {
        let is_draw_invalidated = Arc::new(AtomicBool::new(true));
        let notify_flag = is_draw_invalidated.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    println!("[Watcher] Event: {:?}", event);
                    notify_flag.store(true, Ordering::Relaxed);
                }
                Err(e) => println!("[Watcher] Failed to watch: {:?}", e),
            }
        }).expect("Failed to instantiate file watcher");

        let document_file_path = std::path::Path::new(&archive_path);
        watcher.watch(document_file_path, RecursiveMode::NonRecursive).unwrap();

        let context_settings = ContextSettings::default();
        let mut window = RenderWindow::new(VideoMode::new(1280, 720, 32),
                &format!("{} - {}", uffice_lib::constants::vendor::NAME, document_file_path.file_name().unwrap().to_string_lossy()), Style::DEFAULT, &context_settings);

        window.set_framerate_limit(30);
        window.set_active(true);

        Application {
            archive_path: archive_path.clone(),
            watcher,
            window,
            cursor: Cursor::from_system(CursorType::Arrow).unwrap(),
            keyboard: uffice_lib::Keyboard::new(),

            view: None,

            interface_font: load_interface_font().unwrap(),

            is_draw_invalidated,
            scroller: Scroller::new(),

            zoomer: Zoomer::new(),

            mouse_position: Vector2f::new(0.0, 0.0),
            last_mouse_move: Instant::now(),
            tooltip_state: TooltipState::NotApplicable,
            tooltip_text: String::new(),

            rectangle_shape: RectangleShape::new(),
        }
    }

    pub fn check_interactable_for_mouse(&mut self, mouse_position: Vector2f, callback: &mut dyn FnMut(&mut wp::Node, Position))
            -> bool {
        if let Some(view) = &self.view {
            return view.check_interactable_for_mouse(mouse_position, callback);
        }

        false
    }

    fn display_loading_screen(&mut self) {
        self.window.clear(APPLICATION_BACKGROUND_COLOR);

        let mut text = sfml::graphics::Text::new(&format!("Loading {}", &self.archive_path), &self.interface_font, 36);
        text.set_position((
            (self.window.size().x as f32 - text.local_bounds().width) / 2.0,
            (self.window.size().y as f32 - text.local_bounds().height) / 2.0
        ));

        self.window.draw(&text);
        self.window.display();
    }

    pub fn run(&mut self) {
        let mut shape = sfml::graphics::RectangleShape::new();
        let mut mouse_down = false;
        let mut mouse_position = Vector2f::new(0.0, 0.0);

        let mut current_cursor_type = CursorType::Arrow;
        let mut new_cursor = None;

        let mut page_introduction_animator = Animator::new();

        while self.window.is_open() {
            let window_size = self.window.size();
            {
                while let Some(event) = self.window.poll_event() {
                    self.keyboard.handle_sfml_event(&event);

                    match event {
                        Event::Closed => self.window.close(),
                        Event::Resized { width, height } => {
                            //self.is_draw_invalidated.store(true, Ordering::Relaxed);

                            self.window.set_view(sfml::graphics::View::new(
                                Vector2f::new(
                                    width as f32 / 2.0,
                                    height as f32 / 2.0
                                ),
                                Vector2f::new(
                                    width as f32,
                                    height as f32
                                )
                            ).deref());
                        }
                        Event::KeyPressed { code, alt, ctrl, shift, system } =>
                            self.on_key_pressed(code, alt, ctrl, shift, system),
                        Event::KeyReleased { code, alt, ctrl, shift, system } =>
                            self.on_key_released(code, alt, ctrl, shift, system),

                        Event::MouseWheelScrolled { wheel, delta, x: _, y: _ } => {
                            if wheel == sfml::window::mouse::Wheel::VerticalWheel {
                                if self.keyboard.is_control_key_dow() {
                                    if delta > 0.2 {
                                        self.zoomer.increase_zoom_level();
                                    } else if delta < -0.2 {
                                        self.zoomer.decrease_zoom_level();
                                    }
                                } else  {
                                    self.scroller.scroll(delta);
                                }
                            }
                        }

                        Event::MouseButtonPressed { button, x, y } => {
                            if button == sfml::window::mouse::Button::Left {
                                mouse_position = Vector2f::new(x as f32, y as f32);

                                println!("[ClickEvent] @ {} x {}", x, y);

                                self.scroller.is_hovered = self.scroller.bar_rect.contains(mouse_position);
                                if self.scroller.is_hovered {
                                    self.scroller.is_pressed = true;
                                }

                                if !mouse_down {
                                    mouse_down = true;
                                    // println!("[ClickEvent]   Document Rect @ {} x {}  w {}  h{}", self.document_rect.left, self.document_rect.top,
                                    //         self.document_rect.width, self.document_rect.height);
                                    self.check_interactable_for_mouse(mouse_position, &mut |node, mouse_position| {
                                        node.on_event(&mut wp::Event::Click(MouseEvent::new(mouse_position)));
                                    });
                                }
                            }
                        }
                        Event::MouseButtonReleased { button, x: _, y: _ } => {
                            if button == sfml::window::mouse::Button::Left {
                                mouse_down = false;
                                self.scroller.is_pressed = false;
                            }
                        }
                        Event::MouseMoved { x, y } => {
                            self.scroller.is_hovered = self.scroller.bar_rect.contains(mouse_position);

                            if self.scroller.is_pressed {
                                self.scroller.apply_mouse_offset(y as f32 - mouse_position.y);
                            }

                            mouse_position = Vector2f::new(x as f32, y as f32);
                            new_cursor = Some(CursorType::Arrow);

                            self.reset_tooltip(mouse_position);

                            if let Some(view) = &mut self.view {
                                let mut event = crate::gui::view::Event::MouseMoved(mouse_position, &mut new_cursor);
                                view.handle_event(&mut event);
                            }
                        }
                        _ => (),
                    }
                }
            }

            if let Some(cursor) = new_cursor {
                if cursor != current_cursor_type {
                    current_cursor_type = cursor;
                    self.cursor = Cursor::from_system(cursor).unwrap();
                    unsafe {
                        self.window.set_mouse_cursor(&self.cursor);
                    }
                }
            }

            new_cursor = None;

            // TODO where does this factor come from again? :/
            let factor = 0.6;

            if self.is_draw_invalidated.swap(false, Ordering::Relaxed) {
                self.display_loading_screen();

                // let view = crate::gui::view::View::Document(
                //     crate::gui::view::document_view::DocumentView::new(&self.archive_path)
                // );

                // self.scroller.content_height = view.calculate_content_height() * factor;

                // self.view = Some(view);

                page_introduction_animator.reset();
            }

            self.window.clear(APPLICATION_BACKGROUND_COLOR);
            let zoom_level = self.zoomer.zoom_factor();

            if let Some(view) = &mut self.view {
                view.handle_event(&mut crate::gui::view::Event::Draw(crate::gui::view::DrawEvent{
                    opaqueness: page_introduction_animator.update(),

                    start_y: (VERTICAL_PAGE_MARGIN - self.scroller.content_height * self.scroller.position()) * zoom_level,

                    window: &mut self.window,
                    window_size: window_size,

                    zoom: zoom_level,
                }))
            }

            self.draw_tooltip();

            // Scrollbar
            self.scroller.draw(&mut shape, &mut self.window);

            self.window.display();
        }

        self.window.close();
    }

    fn on_key_pressed(&self, _code: Key, _alt: bool, _ctrl: bool, _shift: bool, _system: bool) {

    }

    fn on_key_released(&mut self, code: Key, _alt: bool, ctrl: bool, _shift: bool, _system: bool) {
        // Control +
        if ctrl && code == Key::Equal {
            self.zoomer.increase_zoom_level();
        }

        // Control -
        if ctrl && code == Key::Hyphen {
            self.zoomer.decrease_zoom_level();
        }

        match code {
            Key::F2 => self.dump_dom_tree(),

            _ => ()
        }
    }

    fn dump_dom_tree(&self) {
        if let Some(view) = &self.view {
            view.dump_dom_tree();
        }
    }

    fn draw_tooltip(&mut self) {
        let now = Instant::now();

        match self.tooltip_state {
            TooltipState::Unchecked => {
                if now.duration_since(self.last_mouse_move) > TOOLTIP_TIMEOUT {
                    let mut tooltip_text = None;
                    let was_hit = self.check_interactable_for_mouse(self.mouse_position, &mut |node, _mouse_position| {
                        match &node.data {
                            wp::NodeData::Hyperlink(link) => {
                                if let Some(url) = link.get_url() {
                                    tooltip_text = Some(url);
                                }
                            }
                            _ => ()
                        }
                    });

                    if !was_hit || tooltip_text.is_none() {
                        self.tooltip_state = TooltipState::NotApplicable;
                        println!("Didn't hit anything @ {:?}", self.mouse_position);
                        return;
                    }

                    self.tooltip_state = TooltipState::Visible;
                    self.tooltip_text = tooltip_text.unwrap();
                }
            }
            TooltipState::NotApplicable => return,
            TooltipState::Visible => ()
        }

        if self.tooltip_text.is_empty() {
            return;
        }

        let mut text = Text::new(&self.tooltip_text, &self.interface_font, 18);
        let text_size = text.global_bounds().size();

        const TOOLTIP_PADDING: f32 = 2.0;
        const TOOLTIP_BORDER: f32 = 2.0;

        self.rectangle_shape.set_size(Vector2f::new(
            text_size.x + TOOLTIP_PADDING * 2.0,
            text_size.y + TOOLTIP_PADDING * 2.0
        ));

        self.rectangle_shape.set_outline_thickness(TOOLTIP_BORDER);
        self.rectangle_shape.set_outline_color(TOOLTIP_BORDER_COLOR);
        self.rectangle_shape.set_fill_color(TOOLTIP_BACKGROUND_COLOR);

        let rectangle_size = self.rectangle_shape.global_bounds().size();
        self.rectangle_shape.set_position(Vector2f::new(
            self.mouse_position.x,
            self.mouse_position.y - rectangle_size.y
        ));

        self.window.draw(&self.rectangle_shape);

        text.set_fill_color(Color::BLACK);

        text.set_position(Vector2f::new(
            self.rectangle_shape.position().x + TOOLTIP_PADDING,
            self.rectangle_shape.position().y,
        ));

        self.window.draw(&text);
    }

    fn reset_tooltip(&mut self, mouse_position: Vector2f) {
        self.mouse_position = mouse_position;
        self.tooltip_state = TooltipState::Unchecked;
        self.last_mouse_move = Instant::now();
        self.tooltip_text = String::new();
    }
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
    path: PathBuf,

    scroller: Scroller,
    zoomer: Zoomer,

    event_loop_proxy: EventLoopProxy<AppEvent>,
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
            event_loop_proxy,
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
    pub fn on_scroll(&mut self, delta: MouseScrollDelta) -> bool {
        if let MouseScrollDelta::LineDelta(_left, top) = delta {
            self.scroller.scroll(top);

            return true;
        }

        return false;
    }
}

pub struct App {
    event_loop_proxy: EventLoopProxy<AppEvent>,

    next_tab_id: usize,
    current_visible_tab: Option<TabId>,
    tabs: HashMap<TabId, Tab>,
}

impl App {
    pub fn new(window: &mut winit::window::Window, event_loop_proxy: EventLoopProxy<AppEvent>, first_file_to_open: String) -> Self {
        window.set_title(&format!("{} - {}", crate::gui::app::formatted_base_title(), first_file_to_open));

        let mut app = Self {
            event_loop_proxy,
            next_tab_id: 1000,
            current_visible_tab: None,
            tabs: HashMap::new(),
        };

        app.add_tab(PathBuf::from(first_file_to_open));

        app
    }

    fn add_tab(&mut self, path: PathBuf) {
        let tab_id = TabId(self.next_tab_id);
        self.next_tab_id += 1;

        self.tabs.insert(tab_id, Tab::new(tab_id, path, self.event_loop_proxy.clone()));

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
}

impl crate::gui::app::GuiApp for App {

    fn on_event(&mut self, window: &mut winit::window::Window, event: winit::event::Event<AppEvent>) {
        use winit::event::Event;
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseWheel { delta }, ..
            } => {
                if let Some(current_tab_id) = self.current_visible_tab {
                    let should_scroll = self.tabs.get_mut(&current_tab_id).unwrap().on_scroll(delta);
                    if should_scroll {
                        window.request_redraw();
                    }
                }
            }

            Event::UserEvent(app_event) => self.handle_user_event(window, app_event),

            _ => ()
        }
    }

    fn paint(&mut self, event: crate::gui::app::PaintEvent) {
        assert!(event.painter.try_borrow_mut().is_ok(), "Failed to painter borrow as mutable; cannot paint App");

        if let Some(current_tab_id) = self.current_visible_tab {
            event.painter.as_ref().borrow_mut().switch_cache(PainterCache::Document(current_tab_id.0));

            let current_tab = self.tabs.get_mut(&current_tab_id).unwrap();
            current_tab.on_paint(&event);

            let mut painter = event.painter.as_ref().borrow_mut();
            painter.switch_cache(PainterCache::UI);

            current_tab.scroller.paint(event.window, &mut *painter);
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
