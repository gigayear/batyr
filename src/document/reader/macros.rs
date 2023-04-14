// Batyr Reader Macros 
// Copyright (C) 2023 Gene Yu
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <https://www.gnu.org/licenses/>.

//! Macros for fetching attribute values from XML start events and
//! converting them to native types

#[allow(unused_macros)]
macro_rules! bool_attr {
    ($event:ident, $name:literal) => {
        {
            let mut value: Option<bool> = None;

            for attr in $event.attributes() {
	        if let Some(attr) = attr.ok() {
                    match attr.key {
		        QName($name) => {
                            if let Ok(s) = str::from_utf8(&attr.value) {
			        match s {
                                    r"true" => {
				        value = Some(true);
                                    },
                                    r"false" => {
				        value = Some(false);
                                    },
                                    _ => (),
			        }
                            }
		        },
		        _ => (),
                    }
	        }
            }

            value
        }
    };
}

macro_rules! char_attr {
    ($event:ident, $name:literal) => {
        {
            let mut value: Option<char> = None;

            for attr in $event.attributes() {
	        if let Some(attr) = attr.ok() {
                    match attr.key {
		        QName($name) => {
                            if let Ok(s) = str::from_utf8(&attr.value) {
                                value = s.chars().next();
                            }
		        },
		        _ => (),
                    }
	        }
            }

            value
        }
    };
}

#[allow(unused_macros)]
macro_rules! enum_attr {
    ($event:ident, $name:literal, $type:ty, $closure:expr) => {
        {
            let mut value: Option<$type> = None;

            for attr in $event.attributes() {
	        if let Some(attr) = attr.ok() {
                    match attr.key {
		        QName($name) => {
                            if let Ok(s) = str::from_utf8(&attr.value) {
                                value = Some($closure(s));
			    }
                        },
		        _ => (),
		    }
                }
	    }

            value
        }
    };
}

macro_rules! numeric_attr {
    ($event:ident, $name:literal, $type:ty) => {
        {
            let mut value: Option<$type> = None;
            
            for attr in $event.attributes() {
	        if let Some(attr) = attr.ok() {
                    match attr.key {
		        QName($name) => {
                            if let Ok(s) = str::from_utf8(&attr.value) {
			        if let Ok(n) = s.parse::<$type>() {
                                    value = Some(n);
			        }
                            }
		        },
		        _ => (),
                    }
	        }
            }

            value
        }
    };
}

#[allow(unused_macros)]
macro_rules! string_attr {
    ($event:ident, $name:literal) => {
        {
            let mut value: Option<String> = None;

            for attr in $event.attributes() {
	        if let Some(attr) = attr.ok() {
                    match attr.key {
		        QName($name) => {
                            if let Ok(s) = str::from_utf8(&attr.value) {
                                value = Some(s.to_string());
                            }
		        },
		        _ => (),
                    }
	        }
            }

            value
        }
    };
}
