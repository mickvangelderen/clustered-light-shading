use glutin::ElementState;
use glutin::VirtualKeyCode;

pub type UncheckedIndex = u8;

const LENGTH: UncheckedIndex = 161;

pub struct Array<T: Sized>([T; LENGTH as usize]);

impl<T> std::ops::Index<Index> for Array<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Index) -> &Self::Output {
        unsafe { self.0.get_unchecked(index.0 as usize) }
    }
}

impl<T> std::ops::IndexMut<Index> for Array<T> {
    #[inline]
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index.0 as usize) }
    }
}

impl<T> std::ops::Deref for Array<T> {
    type Target = [T; LENGTH as usize];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Array<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[repr(transparent)]
pub struct Index(UncheckedIndex);

impl Index {
    pub const INVALID: UncheckedIndex = std::u8::MAX;

    #[inline]
    pub fn new(raw: UncheckedIndex) -> Option<Self> {
        if raw < LENGTH {
            Some(Index(raw))
        } else {
            None
        }
    }

    #[inline]
    pub fn from_code(code: VirtualKeyCode) -> Self {
        match code {
            VirtualKeyCode::Key1 => Index::new(0).unwrap(),
            VirtualKeyCode::Key2 => Index::new(1).unwrap(),
            VirtualKeyCode::Key3 => Index::new(2).unwrap(),
            VirtualKeyCode::Key4 => Index::new(3).unwrap(),
            VirtualKeyCode::Key5 => Index::new(4).unwrap(),
            VirtualKeyCode::Key6 => Index::new(5).unwrap(),
            VirtualKeyCode::Key7 => Index::new(6).unwrap(),
            VirtualKeyCode::Key8 => Index::new(7).unwrap(),
            VirtualKeyCode::Key9 => Index::new(8).unwrap(),
            VirtualKeyCode::Key0 => Index::new(9).unwrap(),
            VirtualKeyCode::A => Index::new(10).unwrap(),
            VirtualKeyCode::B => Index::new(11).unwrap(),
            VirtualKeyCode::C => Index::new(12).unwrap(),
            VirtualKeyCode::D => Index::new(13).unwrap(),
            VirtualKeyCode::E => Index::new(14).unwrap(),
            VirtualKeyCode::F => Index::new(15).unwrap(),
            VirtualKeyCode::G => Index::new(16).unwrap(),
            VirtualKeyCode::H => Index::new(17).unwrap(),
            VirtualKeyCode::I => Index::new(18).unwrap(),
            VirtualKeyCode::J => Index::new(19).unwrap(),
            VirtualKeyCode::K => Index::new(20).unwrap(),
            VirtualKeyCode::L => Index::new(21).unwrap(),
            VirtualKeyCode::M => Index::new(22).unwrap(),
            VirtualKeyCode::N => Index::new(23).unwrap(),
            VirtualKeyCode::O => Index::new(24).unwrap(),
            VirtualKeyCode::P => Index::new(25).unwrap(),
            VirtualKeyCode::Q => Index::new(26).unwrap(),
            VirtualKeyCode::R => Index::new(27).unwrap(),
            VirtualKeyCode::S => Index::new(28).unwrap(),
            VirtualKeyCode::T => Index::new(29).unwrap(),
            VirtualKeyCode::U => Index::new(30).unwrap(),
            VirtualKeyCode::V => Index::new(31).unwrap(),
            VirtualKeyCode::W => Index::new(32).unwrap(),
            VirtualKeyCode::X => Index::new(33).unwrap(),
            VirtualKeyCode::Y => Index::new(34).unwrap(),
            VirtualKeyCode::Z => Index::new(35).unwrap(),
            VirtualKeyCode::Escape => Index::new(36).unwrap(),
            VirtualKeyCode::F1 => Index::new(37).unwrap(),
            VirtualKeyCode::F2 => Index::new(38).unwrap(),
            VirtualKeyCode::F3 => Index::new(39).unwrap(),
            VirtualKeyCode::F4 => Index::new(40).unwrap(),
            VirtualKeyCode::F5 => Index::new(41).unwrap(),
            VirtualKeyCode::F6 => Index::new(42).unwrap(),
            VirtualKeyCode::F7 => Index::new(43).unwrap(),
            VirtualKeyCode::F8 => Index::new(44).unwrap(),
            VirtualKeyCode::F9 => Index::new(45).unwrap(),
            VirtualKeyCode::F10 => Index::new(46).unwrap(),
            VirtualKeyCode::F11 => Index::new(47).unwrap(),
            VirtualKeyCode::F12 => Index::new(48).unwrap(),
            VirtualKeyCode::F13 => Index::new(49).unwrap(),
            VirtualKeyCode::F14 => Index::new(50).unwrap(),
            VirtualKeyCode::F15 => Index::new(51).unwrap(),
            VirtualKeyCode::F16 => Index::new(52).unwrap(),
            VirtualKeyCode::F17 => Index::new(53).unwrap(),
            VirtualKeyCode::F18 => Index::new(54).unwrap(),
            VirtualKeyCode::F19 => Index::new(55).unwrap(),
            VirtualKeyCode::F20 => Index::new(56).unwrap(),
            VirtualKeyCode::F21 => Index::new(57).unwrap(),
            VirtualKeyCode::F22 => Index::new(58).unwrap(),
            VirtualKeyCode::F23 => Index::new(59).unwrap(),
            VirtualKeyCode::F24 => Index::new(60).unwrap(),
            VirtualKeyCode::Snapshot => Index::new(61).unwrap(),
            VirtualKeyCode::Scroll => Index::new(62).unwrap(),
            VirtualKeyCode::Pause => Index::new(63).unwrap(),
            VirtualKeyCode::Insert => Index::new(64).unwrap(),
            VirtualKeyCode::Home => Index::new(65).unwrap(),
            VirtualKeyCode::Delete => Index::new(66).unwrap(),
            VirtualKeyCode::End => Index::new(67).unwrap(),
            VirtualKeyCode::PageDown => Index::new(68).unwrap(),
            VirtualKeyCode::PageUp => Index::new(69).unwrap(),
            VirtualKeyCode::Left => Index::new(70).unwrap(),
            VirtualKeyCode::Up => Index::new(71).unwrap(),
            VirtualKeyCode::Right => Index::new(72).unwrap(),
            VirtualKeyCode::Down => Index::new(73).unwrap(),
            VirtualKeyCode::Back => Index::new(74).unwrap(),
            VirtualKeyCode::Return => Index::new(75).unwrap(),
            VirtualKeyCode::Space => Index::new(76).unwrap(),
            VirtualKeyCode::Compose => Index::new(77).unwrap(),
            VirtualKeyCode::Caret => Index::new(78).unwrap(),
            VirtualKeyCode::Numlock => Index::new(79).unwrap(),
            VirtualKeyCode::Numpad0 => Index::new(80).unwrap(),
            VirtualKeyCode::Numpad1 => Index::new(81).unwrap(),
            VirtualKeyCode::Numpad2 => Index::new(82).unwrap(),
            VirtualKeyCode::Numpad3 => Index::new(83).unwrap(),
            VirtualKeyCode::Numpad4 => Index::new(84).unwrap(),
            VirtualKeyCode::Numpad5 => Index::new(85).unwrap(),
            VirtualKeyCode::Numpad6 => Index::new(86).unwrap(),
            VirtualKeyCode::Numpad7 => Index::new(87).unwrap(),
            VirtualKeyCode::Numpad8 => Index::new(88).unwrap(),
            VirtualKeyCode::Numpad9 => Index::new(89).unwrap(),
            VirtualKeyCode::AbntC1 => Index::new(90).unwrap(),
            VirtualKeyCode::AbntC2 => Index::new(91).unwrap(),
            VirtualKeyCode::Add => Index::new(92).unwrap(),
            VirtualKeyCode::Apostrophe => Index::new(93).unwrap(),
            VirtualKeyCode::Apps => Index::new(94).unwrap(),
            VirtualKeyCode::At => Index::new(95).unwrap(),
            VirtualKeyCode::Ax => Index::new(96).unwrap(),
            VirtualKeyCode::Backslash => Index::new(97).unwrap(),
            VirtualKeyCode::Calculator => Index::new(98).unwrap(),
            VirtualKeyCode::Capital => Index::new(99).unwrap(),
            VirtualKeyCode::Colon => Index::new(100).unwrap(),
            VirtualKeyCode::Comma => Index::new(101).unwrap(),
            VirtualKeyCode::Convert => Index::new(102).unwrap(),
            VirtualKeyCode::Decimal => Index::new(103).unwrap(),
            VirtualKeyCode::Divide => Index::new(104).unwrap(),
            VirtualKeyCode::Equals => Index::new(105).unwrap(),
            VirtualKeyCode::Grave => Index::new(106).unwrap(),
            VirtualKeyCode::Kana => Index::new(107).unwrap(),
            VirtualKeyCode::Kanji => Index::new(108).unwrap(),
            VirtualKeyCode::LAlt => Index::new(109).unwrap(),
            VirtualKeyCode::LBracket => Index::new(110).unwrap(),
            VirtualKeyCode::LControl => Index::new(111).unwrap(),
            VirtualKeyCode::LShift => Index::new(112).unwrap(),
            VirtualKeyCode::LWin => Index::new(113).unwrap(),
            VirtualKeyCode::Mail => Index::new(114).unwrap(),
            VirtualKeyCode::MediaSelect => Index::new(115).unwrap(),
            VirtualKeyCode::MediaStop => Index::new(116).unwrap(),
            VirtualKeyCode::Minus => Index::new(117).unwrap(),
            VirtualKeyCode::Multiply => Index::new(118).unwrap(),
            VirtualKeyCode::Mute => Index::new(119).unwrap(),
            VirtualKeyCode::MyComputer => Index::new(120).unwrap(),
            VirtualKeyCode::NavigateForward => Index::new(121).unwrap(),
            VirtualKeyCode::NavigateBackward => Index::new(122).unwrap(),
            VirtualKeyCode::NextTrack => Index::new(123).unwrap(),
            VirtualKeyCode::NoConvert => Index::new(124).unwrap(),
            VirtualKeyCode::NumpadComma => Index::new(125).unwrap(),
            VirtualKeyCode::NumpadEnter => Index::new(126).unwrap(),
            VirtualKeyCode::NumpadEquals => Index::new(127).unwrap(),
            VirtualKeyCode::OEM102 => Index::new(128).unwrap(),
            VirtualKeyCode::Period => Index::new(129).unwrap(),
            VirtualKeyCode::PlayPause => Index::new(130).unwrap(),
            VirtualKeyCode::Power => Index::new(131).unwrap(),
            VirtualKeyCode::PrevTrack => Index::new(132).unwrap(),
            VirtualKeyCode::RAlt => Index::new(133).unwrap(),
            VirtualKeyCode::RBracket => Index::new(134).unwrap(),
            VirtualKeyCode::RControl => Index::new(135).unwrap(),
            VirtualKeyCode::RShift => Index::new(136).unwrap(),
            VirtualKeyCode::RWin => Index::new(137).unwrap(),
            VirtualKeyCode::Semicolon => Index::new(138).unwrap(),
            VirtualKeyCode::Slash => Index::new(139).unwrap(),
            VirtualKeyCode::Sleep => Index::new(140).unwrap(),
            VirtualKeyCode::Stop => Index::new(141).unwrap(),
            VirtualKeyCode::Subtract => Index::new(142).unwrap(),
            VirtualKeyCode::Sysrq => Index::new(143).unwrap(),
            VirtualKeyCode::Tab => Index::new(144).unwrap(),
            VirtualKeyCode::Underline => Index::new(145).unwrap(),
            VirtualKeyCode::Unlabeled => Index::new(146).unwrap(),
            VirtualKeyCode::VolumeDown => Index::new(147).unwrap(),
            VirtualKeyCode::VolumeUp => Index::new(148).unwrap(),
            VirtualKeyCode::Wake => Index::new(149).unwrap(),
            VirtualKeyCode::WebBack => Index::new(150).unwrap(),
            VirtualKeyCode::WebFavorites => Index::new(151).unwrap(),
            VirtualKeyCode::WebForward => Index::new(152).unwrap(),
            VirtualKeyCode::WebHome => Index::new(153).unwrap(),
            VirtualKeyCode::WebRefresh => Index::new(154).unwrap(),
            VirtualKeyCode::WebSearch => Index::new(155).unwrap(),
            VirtualKeyCode::WebStop => Index::new(156).unwrap(),
            VirtualKeyCode::Yen => Index::new(157).unwrap(),
            VirtualKeyCode::Copy => Index::new(158).unwrap(),
            VirtualKeyCode::Paste => Index::new(159).unwrap(),
            VirtualKeyCode::Cut => Index::new(160).unwrap(),
        }
    }
}

impl From<Index> for UncheckedIndex {
    #[inline]
    fn from(val: Index) -> Self {
        val.0
    }
}

pub struct KeyboardModel {
    hard_states: Array<ElementState>,
    soft_states: Array<f32>,
}

impl KeyboardModel {
    pub fn new() -> Self {
        KeyboardModel {
            hard_states: Array([ElementState::Released; LENGTH as usize]),
            soft_states: Array([0.0; LENGTH as usize]),
        }
    }

    pub fn process_event(&mut self, code: VirtualKeyCode, state: ElementState) {
        self.hard_states[Index::from_code(code)] = state;
    }

    pub fn simulate(&mut self, delta_time: f32) {
        for (s, h) in self.soft_states.iter_mut().zip(self.hard_states.iter()) {
            match h {
                ElementState::Pressed => {
                    *s += 40.0 * delta_time;
                    if *s > 1.0 {
                        *s = 1.0;
                    }
                }
                ElementState::Released => {
                    *s -= 15.0 * delta_time;
                    if *s < 0.0 {
                        *s = 0.0;
                    }
                }
            }
        }
    }

    #[inline]
    pub fn pressure(&self, index: Index) -> f32 {
        self.soft_states[index]
    }
}
