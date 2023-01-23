// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::string::FromUtf8Error;

use windows::{
    core::{PCSTR, PSTR},
    Win32::{
        System::Registry::{
            HKEY,
            HKEY_CLASSES_ROOT,
            HKEY_CURRENT_CONFIG,
            HKEY_CURRENT_USER,
            HKEY_CURRENT_USER_LOCAL_SETTINGS,
            HKEY_LOCAL_MACHINE,
            HKEY_PERFORMANCE_DATA,
            HKEY_PERFORMANCE_NLSTEXT,
            HKEY_PERFORMANCE_TEXT,
            HKEY_USERS,
            KEY_READ,
            RegCloseKey,
            RegEnumValueA,
            RegOpenKeyExA,
            RegQueryInfoKeyA,
            REG_BINARY,
            REG_DWORD,
            REG_DWORD_BIG_ENDIAN,
            REG_EXPAND_SZ,
            REG_LINK,
            REG_MULTI_SZ,
            REG_NONE,
            REG_QWORD,
            REG_SZ,
            REG_VALUE_TYPE,
        },
        Foundation::{
            ERROR_SUCCESS, ERROR_MORE_DATA, ERROR_FILE_NOT_FOUND
        },
    }
};

#[derive(Debug)]
pub enum RegistryError {
    KeyNotFound,
    Unknown(u32),
    Utf8Error(FromUtf8Error),
    UnknownRegistryValueType(u32),
    UnexpectedDataType,
}

impl From<FromUtf8Error> for RegistryError {
    fn from(value: FromUtf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

/// [Win32 Documentation](https://learn.microsoft.com/en-us/windows/win32/sysinfo/predefined-keys).
#[derive(Debug)]
pub enum PredefinedRegistryKey {
    ClassesRoot,
    CurrentConfig,
    CurrentUser,
    CurrentUserLocalSettings,
    LocalMachine,
    PerformanceData,
    PerformanceNlsText,
    PerformanceText,
    Users,
}

impl From<PredefinedRegistryKey> for HKEY {
    fn from(value: PredefinedRegistryKey) -> Self {
        match value {
            PredefinedRegistryKey::ClassesRoot => HKEY_CLASSES_ROOT,
            PredefinedRegistryKey::CurrentConfig => HKEY_CURRENT_CONFIG,
            PredefinedRegistryKey::CurrentUser => HKEY_CURRENT_USER,
            PredefinedRegistryKey::CurrentUserLocalSettings => HKEY_CURRENT_USER_LOCAL_SETTINGS,
            PredefinedRegistryKey::LocalMachine => HKEY_LOCAL_MACHINE,
            PredefinedRegistryKey::PerformanceData => HKEY_PERFORMANCE_DATA,
            PredefinedRegistryKey::PerformanceNlsText => HKEY_PERFORMANCE_NLSTEXT,
            PredefinedRegistryKey::PerformanceText => HKEY_PERFORMANCE_TEXT,
            PredefinedRegistryKey::Users => HKEY_USERS,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegistryKeyInformation {
    pub sub_key_count: u32,
    pub value_count: u32,
    pub max_subkey_length: u32,
    pub max_subkey_class_length: u32,
    pub max_value_name_length: u32,
    pub max_value_length: u32,
}

/// [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
#[derive(Debug)]
pub enum RegistryValueData {
    Binary(Vec<u8>),
    Dword(i32),
    UnexpandedString(String),
    Link,
    MultiString(Vec<String>),
    None,
    Qword(i64),
    String(String),
}

impl RegistryValueData {
    pub fn as_str(&self) -> Result<&str, RegistryError> {
        match self {
            Self::String(s) => Ok(&s),
            _ => Err(RegistryError::UnexpectedDataType),
        }
    }
}

impl RegistryValueData {
    pub fn convert(mut value: Vec<u8>, value_type: u32) -> Result<Self, RegistryError> {
        match REG_VALUE_TYPE(value_type) {
            REG_BINARY => Ok(Self::Binary(value)),
            REG_DWORD => Ok(Self::Dword(i32::from_ne_bytes(value.try_into().unwrap()))),
            REG_DWORD_BIG_ENDIAN => Ok(Self::Dword(i32::from_be_bytes(value.try_into().unwrap()))),
            REG_EXPAND_SZ => todo!(),
            REG_LINK => todo!(),
            REG_MULTI_SZ => todo!(),
            REG_NONE => Ok(Self::None),
            REG_QWORD => Ok(Self::Qword(i64::from_ne_bytes(value.try_into().unwrap()))),
            REG_SZ => Ok(Self::String(unsafe {
                PSTR(value.as_mut_ptr()).to_string()?
            })),
            _ => Err(RegistryError::UnknownRegistryValueType(value_type))
        }
    }
}

#[derive(Debug)]
pub struct RegistryValue {
    pub name: String,
    pub data: RegistryValueData,
}

pub struct RegistryKey {
    handle: HKEY,
}

#[allow(dead_code)]
impl RegistryKey {

    pub fn open(key: PredefinedRegistryKey) -> Result<Self, RegistryError> {
        let mut handle: HKEY = Default::default();
        let result = unsafe { RegOpenKeyExA(key, None, 0, KEY_READ, &mut handle) };

        match result {
            ERROR_SUCCESS => Ok(Self{ handle }),
            ERROR_FILE_NOT_FOUND => Err(RegistryError::KeyNotFound),
            _ => Err(RegistryError::Unknown(result.0))
        }
    }

    pub fn info(&self) -> RegistryKeyInformation {
        let mut info: RegistryKeyInformation = Default::default();
        unsafe {
            RegQueryInfoKeyA(self.handle,
                windows::core::PSTR(std::ptr::null_mut() as *mut _),
                None,
                None,
                Some(&mut info.sub_key_count),
                Some(&mut info.max_subkey_length),
                Some(&mut info.max_subkey_class_length),
                Some(&mut info.value_count),
                Some(&mut info.max_value_name_length),
                Some(&mut info.max_value_length),
                None,
                None);
        }

        info
    }

    pub fn open_subkey(&self, name: &str) -> Result<RegistryKey, RegistryError> {
        let mut handle: HKEY = Default::default();
        let result = unsafe { RegOpenKeyExA(self.handle, Some(PCSTR(name.as_ptr())), 0, KEY_READ, &mut handle) };

        match result {
            ERROR_SUCCESS => Ok(Self{ handle }),
            ERROR_FILE_NOT_FOUND => Err(RegistryError::KeyNotFound),
            _ => Err(RegistryError::Unknown(result.0))
        }
    }

    pub fn value_by_index(&self, index: u32) -> Result<RegistryValue, RegistryError> {
        let mut name = Vec::new();
        name.resize(256, 0);

        let mut name_length = name.len() as u32 - 1;
        let name_str = PSTR(name.as_mut_ptr());

        let mut value = Vec::new();
        value.resize(255, 0);

        let mut value_length = value.len() as u32;

        let mut value_type = 0;

        let mut result = unsafe {
            RegEnumValueA(self.handle,
                index,
                name_str,
                &mut name_length,
                None,
                Some(&mut value_type),
                Some(value.as_mut_ptr()),
                Some(&mut value_length))
        };

        value.resize(value_length as usize, 0);

        if result == ERROR_MORE_DATA {
            name.resize(name_length as usize + 1, 0);

            unsafe {
                result = RegEnumValueA(self.handle,
                    index,
                    name_str,
                    &mut name_length,
                    None,
                    Some(&mut value_type),
                    Some(value.as_mut_ptr()),
                    Some(&mut value_length))
            }
        }

        if result != ERROR_SUCCESS {
            Err(RegistryError::Unknown(result.0))
        } else {
            Ok(RegistryValue {
                name: unsafe {
                    name_str.to_string()?
                },
                data: RegistryValueData::convert(value, value_type)?,
            })
        }
    }

    pub fn values(&self) -> Result<Vec<RegistryValue>, RegistryError> {
        let info = self.info();
        let mut result = Vec::new();
        for i in 0..info.value_count {
            result.push(self.value_by_index(i)?);
        }
        Ok(result)
    }

}

impl Drop for RegistryKey {
    fn drop(&mut self) {
        if self.handle.is_invalid() {
            return;
        }

        unsafe {
            let result = RegCloseKey(self.handle);
            assert!(result == ERROR_SUCCESS, "RegCloseKey failed: {:?}", result);
        }
    }
}
