// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub mod vendor {

    /// The name of the vendor.
    /// This constant is used for the ProgID.
    pub const NAME: &'static str = "TheWoosh";

}

pub mod component {

    macro_rules! prog_id {
        ($name:expr, $version:expr) => {
            {
                const_format::concatcp!(crate::constants::vendor::NAME, ".", $name, ".", $version)
            }
        };
    }

    pub mod document {

        /// The name of the 'Document' component (file type).
        /// This constant is used for the ProgID.
        pub const NAME: &'static str = "Document";

        pub const VERSION: u32 = 1;

        /// https://learn.microsoft.com/en-us/windows/win32/shell/fa-progids
        pub const PROG_ID: &'static str = prog_id!(NAME, VERSION);

        pub const FRIENDLY_INTERNATIONAL_NAME: &'static str = "TheWoosh Uffice Document";
    }

}


