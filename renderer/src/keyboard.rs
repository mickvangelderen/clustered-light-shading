macro_rules! implement_with_keys {
    ($(($key: ident, $Key: ident),)*) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub enum Key {
            $(
                $Key,
            )*
        }

        impl Key {
            #[inline]
            pub fn from_virtual_key_code(code: glutin::VirtualKeyCode) -> Option<Self> {
                match code {
                    $(
                        glutin::VirtualKeyCode::$Key => Some(Key::$Key),
                    )*
                    _ => None,
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct KeyboardState {
            $(
                pub $key: glutin::ElementState,
            )*
        }

        impl KeyboardState {
            #[inline]
            pub fn update(&mut self, update: glutin::KeyboardInput) {
                let glutin::KeyboardInput { virtual_keycode, state, .. } = update;
                if let Some(key) = virtual_keycode.and_then(Key::from_virtual_key_code) {
                    self[key] = state;
                }
            }
        }

        impl Default for KeyboardState {
            #[inline]
            fn default() -> Self {
                KeyboardState {
                    $(
                        $key: glutin::ElementState::Released,
                    )*
                }
            }
        }


        impl std::ops::Index<Key> for KeyboardState {
            type Output = glutin::ElementState;

            #[inline]
            fn index<'a>(&'a self, index: Key) -> &'a Self::Output {
                match index {
                    $(
                        Key::$Key => &self.$key,
                    )*
                }
            }
        }

        impl std::ops::IndexMut<Key> for KeyboardState {
            #[inline]
            fn index_mut<'a>(&'a mut self, index: Key) -> &'a mut glutin::ElementState {
                match index {
                    $(
                        Key::$Key => &mut self.$key,
                    )*
                }
            }
        }
    }
}

implement_with_keys!(
    (w, W),
    (s, S),
    (d, D),
    (a, A),
    (q, Q),
    (z, Z),
    (p, P),
    (up, Up),
    (down, Down),
    (right, Right),
    (left, Left),
    (lshift, LShift),
);
