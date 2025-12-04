// Gupax
//
// Copyright (c) 2024-2025 Cyrix126
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// These are general QoL macros, nothing too scary, I promise.
//
// | MACRO   | PURPOSE                                       | EQUIVALENT CODE                                            |
// |---------|-----------------------------------------------|------------------------------------------------------------|
// | arc_mut | Create a new [Arc<Mutex>]                     | std::sync::Arc::new(std::sync::Mutex::new(my_value))       |
// | sleep   | Sleep the current thread for x milliseconds   | std::thread::sleep(std::time::Duration::from_millis(1000)) |
// | flip    | Flip a bool in place                          | my_bool = !my_bool                                         |
//

// Creates a new [Arc<Mutex<T>]
macro_rules! arc_mut {
    ($arc_mutex:expr) => {
        std::sync::Arc::new(std::sync::Mutex::new($arc_mutex))
    };
}
pub(crate) use arc_mut;

// Sleeps a [std::thread] using milliseconds
macro_rules! sleep {
    ($millis:expr) => {
        std::thread::sleep(std::time::Duration::from_millis($millis))
    };
}
pub(crate) use sleep;

// Flips a [bool] in place
macro_rules! flip {
    ($b:expr) => {
        match $b {
            true | false => $b = !$b,
        }
    };
}
pub(crate) use flip;

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {

    #[test]
    fn arc_mut() {
        let a = arc_mut!(false);
        assert!(!(*a.lock().unwrap()));
    }
    #[test]
    fn flip() {
        let mut b = true;
        flip!(b);
        assert!(!b);
    }
}
