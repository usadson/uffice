// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

#[derive(Debug)]
pub enum SettingState<T> {
    /// Automatic and follows the system setting wherever possible.
    Default(T),

    /// The user changed the option.
    #[allow(dead_code)] // TODO: implement settings menu
    Manual(T),
}

impl<T> SettingState<T> {
    fn get(&self) -> &T {
        match self {
            Self::Default(value) => value,
            Self::Manual(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingName {
    /// Whether or not to enable animations. These may be disabled as a measure
    /// for accessibility.
    EnableAnimations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum SettingChangeOrigin {
    /// The user changed this setting in the application.
    User,

    /// A system parameter or policy was changed. This can also be a (direct)
    /// result of the action of the user, but is received as an event by the
    /// operating system.
    System,
}

#[derive(Debug)]
/// Information about a setting that was changed.
pub struct SettingChangeNotification<'a> {
    /// Indicating where the change originated from.
    pub origin: SettingChangeOrigin,

    /// The name of the setting that was changed.
    pub setting_name: SettingName,

    /// A reference to the settings which can be used to update some part of
    /// the object that was notified.
    pub settings: &'a UserSettings,
}

/// A trait indicating that the class be notified about changes in settings.
pub trait SettingChangeSubscriber {
    /// Called when the settings are loaded at the beginning of the application
    /// execution, or the beginning of the component initialization.
    fn settings_loaded(&mut self, settings: &UserSettings);

    /// Called when a setting was changed.
    fn setting_changed(&mut self, notification: &SettingChangeNotification);
}

impl<T: Default> Default for SettingState<T> {
    fn default() -> Self {
        Self::Default(Default::default())
    }
}

#[derive(Default, Debug)]
pub struct UserSettings {
    /// Whether or not to enable animations. These may be disabled as a measure
    /// for accessibility.
    enable_animations: SettingState<bool>,
}

impl UserSettings {

    pub fn load() -> Self {
        let mut settings: Self = Default::default();
        settings.reload_system_settings();
        settings
    }

    #[cfg(windows)]
    /// Loads the `Default` settings from the system.
    pub fn reload_system_settings(&mut self) {
        use std::ffi::c_void;

        use windows::Win32::{UI::WindowsAndMessaging::{
            SystemParametersInfoA,
            SPI_GETCLIENTAREAANIMATION
        }, Foundation::BOOL};

        let mut value: BOOL = true.into();
        unsafe {
            let ptr = &mut value as *mut BOOL as *mut c_void;
            SystemParametersInfoA(SPI_GETCLIENTAREAANIMATION, 0, Some(ptr), Default::default());
        }
        self.enable_animations = SettingState::Default(value.into());
    }

    #[cfg(not(windows))]
    /// Loads the `Default` settings from the system.
    pub fn reload_system_settings(&mut self) {
        println!("[UserSettings] TODO: reload_system_settings()");
    }

    /// Whether or not to enable animations. These may be disabled as a measure
    /// for accessibility.
    pub fn setting_enable_animations(&self) -> bool {
        *self.enable_animations.get()
    }

}

