use std::num::NonZeroU8;
use glutin::VirtualKeyCode;
use glutin::ElementState;

pub struct KeyboardModel {
    hard_states: [ElementState; 161],
    soft_states: [f32; 161],
}

impl KeyboardModel {
    pub fn new() -> Self {
        KeyboardModel {
            hard_states: [ElementState::Released; 161],
            soft_states: [0.0; 161],
        }
    }

    pub fn process_event(&mut self, code: VirtualKeyCode, state: ElementState) {
        self.hard_states[Self::code_to_index(code).get() as usize] = state;
    }

    pub fn simulate(&mut self, delta_time: f32) {
        for (s, h) in self.soft_states.iter_mut().zip(self.hard_states.iter()) {
            match h {
                ElementState::Pressed => {
                    *s += 40.0 * delta_time;
                    if *s > 1.0 {
                        *s = 1.0;
                    }
                },
                ElementState::Released => {
                    *s -= 15.0 * delta_time;
                    if *s < 0.0 {
                        *s = 0.0;
                    }
                },
            }
        }
    }

    #[inline]
    pub fn pressure(&self, index: NonZeroU8) -> f32 {
        self.soft_states[index.get() as usize]
    }

    #[inline]
    pub fn code_to_index(code: VirtualKeyCode) -> NonZeroU8 {
        match code {
            VirtualKeyCode::Key1 => NonZeroU8::new(1).unwrap(),
            VirtualKeyCode::Key2 => NonZeroU8::new(2).unwrap(),
            VirtualKeyCode::Key3 => NonZeroU8::new(3).unwrap(),
            VirtualKeyCode::Key4 => NonZeroU8::new(4).unwrap(),
            VirtualKeyCode::Key5 => NonZeroU8::new(5).unwrap(),
            VirtualKeyCode::Key6 => NonZeroU8::new(6).unwrap(),
            VirtualKeyCode::Key7 => NonZeroU8::new(7).unwrap(),
            VirtualKeyCode::Key8 => NonZeroU8::new(8).unwrap(),
            VirtualKeyCode::Key9 => NonZeroU8::new(9).unwrap(),
            VirtualKeyCode::Key0 => NonZeroU8::new(10).unwrap(),
            VirtualKeyCode::A => NonZeroU8::new(11).unwrap(),
            VirtualKeyCode::B => NonZeroU8::new(12).unwrap(),
            VirtualKeyCode::C => NonZeroU8::new(13).unwrap(),
            VirtualKeyCode::D => NonZeroU8::new(14).unwrap(),
            VirtualKeyCode::E => NonZeroU8::new(15).unwrap(),
            VirtualKeyCode::F => NonZeroU8::new(16).unwrap(),
            VirtualKeyCode::G => NonZeroU8::new(17).unwrap(),
            VirtualKeyCode::H => NonZeroU8::new(18).unwrap(),
            VirtualKeyCode::I => NonZeroU8::new(19).unwrap(),
            VirtualKeyCode::J => NonZeroU8::new(20).unwrap(),
            VirtualKeyCode::K => NonZeroU8::new(21).unwrap(),
            VirtualKeyCode::L => NonZeroU8::new(22).unwrap(),
            VirtualKeyCode::M => NonZeroU8::new(23).unwrap(),
            VirtualKeyCode::N => NonZeroU8::new(24).unwrap(),
            VirtualKeyCode::O => NonZeroU8::new(25).unwrap(),
            VirtualKeyCode::P => NonZeroU8::new(26).unwrap(),
            VirtualKeyCode::Q => NonZeroU8::new(27).unwrap(),
            VirtualKeyCode::R => NonZeroU8::new(28).unwrap(),
            VirtualKeyCode::S => NonZeroU8::new(29).unwrap(),
            VirtualKeyCode::T => NonZeroU8::new(30).unwrap(),
            VirtualKeyCode::U => NonZeroU8::new(31).unwrap(),
            VirtualKeyCode::V => NonZeroU8::new(32).unwrap(),
            VirtualKeyCode::W => NonZeroU8::new(33).unwrap(),
            VirtualKeyCode::X => NonZeroU8::new(34).unwrap(),
            VirtualKeyCode::Y => NonZeroU8::new(35).unwrap(),
            VirtualKeyCode::Z => NonZeroU8::new(36).unwrap(),
            VirtualKeyCode::Escape => NonZeroU8::new(37).unwrap(),
            VirtualKeyCode::F1 => NonZeroU8::new(38).unwrap(),
            VirtualKeyCode::F2 => NonZeroU8::new(39).unwrap(),
            VirtualKeyCode::F3 => NonZeroU8::new(40).unwrap(),
            VirtualKeyCode::F4 => NonZeroU8::new(41).unwrap(),
            VirtualKeyCode::F5 => NonZeroU8::new(42).unwrap(),
            VirtualKeyCode::F6 => NonZeroU8::new(43).unwrap(),
            VirtualKeyCode::F7 => NonZeroU8::new(44).unwrap(),
            VirtualKeyCode::F8 => NonZeroU8::new(45).unwrap(),
            VirtualKeyCode::F9 => NonZeroU8::new(46).unwrap(),
            VirtualKeyCode::F10 => NonZeroU8::new(47).unwrap(),
            VirtualKeyCode::F11 => NonZeroU8::new(48).unwrap(),
            VirtualKeyCode::F12 => NonZeroU8::new(49).unwrap(),
            VirtualKeyCode::F13 => NonZeroU8::new(50).unwrap(),
            VirtualKeyCode::F14 => NonZeroU8::new(51).unwrap(),
            VirtualKeyCode::F15 => NonZeroU8::new(52).unwrap(),
            VirtualKeyCode::F16 => NonZeroU8::new(53).unwrap(),
            VirtualKeyCode::F17 => NonZeroU8::new(54).unwrap(),
            VirtualKeyCode::F18 => NonZeroU8::new(55).unwrap(),
            VirtualKeyCode::F19 => NonZeroU8::new(56).unwrap(),
            VirtualKeyCode::F20 => NonZeroU8::new(57).unwrap(),
            VirtualKeyCode::F21 => NonZeroU8::new(58).unwrap(),
            VirtualKeyCode::F22 => NonZeroU8::new(59).unwrap(),
            VirtualKeyCode::F23 => NonZeroU8::new(60).unwrap(),
            VirtualKeyCode::F24 => NonZeroU8::new(61).unwrap(),
            VirtualKeyCode::Snapshot => NonZeroU8::new(62).unwrap(),
            VirtualKeyCode::Scroll => NonZeroU8::new(63).unwrap(),
            VirtualKeyCode::Pause => NonZeroU8::new(64).unwrap(),
            VirtualKeyCode::Insert => NonZeroU8::new(65).unwrap(),
            VirtualKeyCode::Home => NonZeroU8::new(66).unwrap(),
            VirtualKeyCode::Delete => NonZeroU8::new(67).unwrap(),
            VirtualKeyCode::End => NonZeroU8::new(68).unwrap(),
            VirtualKeyCode::PageDown => NonZeroU8::new(69).unwrap(),
            VirtualKeyCode::PageUp => NonZeroU8::new(70).unwrap(),
            VirtualKeyCode::Left => NonZeroU8::new(71).unwrap(),
            VirtualKeyCode::Up => NonZeroU8::new(72).unwrap(),
            VirtualKeyCode::Right => NonZeroU8::new(73).unwrap(),
            VirtualKeyCode::Down => NonZeroU8::new(74).unwrap(),
            VirtualKeyCode::Back => NonZeroU8::new(75).unwrap(),
            VirtualKeyCode::Return => NonZeroU8::new(76).unwrap(),
            VirtualKeyCode::Space => NonZeroU8::new(77).unwrap(),
            VirtualKeyCode::Compose => NonZeroU8::new(78).unwrap(),
            VirtualKeyCode::Caret => NonZeroU8::new(79).unwrap(),
            VirtualKeyCode::Numlock => NonZeroU8::new(80).unwrap(),
            VirtualKeyCode::Numpad0 => NonZeroU8::new(81).unwrap(),
            VirtualKeyCode::Numpad1 => NonZeroU8::new(82).unwrap(),
            VirtualKeyCode::Numpad2 => NonZeroU8::new(83).unwrap(),
            VirtualKeyCode::Numpad3 => NonZeroU8::new(84).unwrap(),
            VirtualKeyCode::Numpad4 => NonZeroU8::new(85).unwrap(),
            VirtualKeyCode::Numpad5 => NonZeroU8::new(86).unwrap(),
            VirtualKeyCode::Numpad6 => NonZeroU8::new(87).unwrap(),
            VirtualKeyCode::Numpad7 => NonZeroU8::new(88).unwrap(),
            VirtualKeyCode::Numpad8 => NonZeroU8::new(89).unwrap(),
            VirtualKeyCode::Numpad9 => NonZeroU8::new(90).unwrap(),
            VirtualKeyCode::AbntC1 => NonZeroU8::new(91).unwrap(),
            VirtualKeyCode::AbntC2 => NonZeroU8::new(92).unwrap(),
            VirtualKeyCode::Add => NonZeroU8::new(93).unwrap(),
            VirtualKeyCode::Apostrophe => NonZeroU8::new(94).unwrap(),
            VirtualKeyCode::Apps => NonZeroU8::new(95).unwrap(),
            VirtualKeyCode::At => NonZeroU8::new(96).unwrap(),
            VirtualKeyCode::Ax => NonZeroU8::new(97).unwrap(),
            VirtualKeyCode::Backslash => NonZeroU8::new(98).unwrap(),
            VirtualKeyCode::Calculator => NonZeroU8::new(99).unwrap(),
            VirtualKeyCode::Capital => NonZeroU8::new(100).unwrap(),
            VirtualKeyCode::Colon => NonZeroU8::new(101).unwrap(),
            VirtualKeyCode::Comma => NonZeroU8::new(102).unwrap(),
            VirtualKeyCode::Convert => NonZeroU8::new(103).unwrap(),
            VirtualKeyCode::Decimal => NonZeroU8::new(104).unwrap(),
            VirtualKeyCode::Divide => NonZeroU8::new(105).unwrap(),
            VirtualKeyCode::Equals => NonZeroU8::new(106).unwrap(),
            VirtualKeyCode::Grave => NonZeroU8::new(107).unwrap(),
            VirtualKeyCode::Kana => NonZeroU8::new(108).unwrap(),
            VirtualKeyCode::Kanji => NonZeroU8::new(109).unwrap(),
            VirtualKeyCode::LAlt => NonZeroU8::new(110).unwrap(),
            VirtualKeyCode::LBracket => NonZeroU8::new(111).unwrap(),
            VirtualKeyCode::LControl => NonZeroU8::new(112).unwrap(),
            VirtualKeyCode::LShift => NonZeroU8::new(113).unwrap(),
            VirtualKeyCode::LWin => NonZeroU8::new(114).unwrap(),
            VirtualKeyCode::Mail => NonZeroU8::new(115).unwrap(),
            VirtualKeyCode::MediaSelect => NonZeroU8::new(116).unwrap(),
            VirtualKeyCode::MediaStop => NonZeroU8::new(117).unwrap(),
            VirtualKeyCode::Minus => NonZeroU8::new(118).unwrap(),
            VirtualKeyCode::Multiply => NonZeroU8::new(119).unwrap(),
            VirtualKeyCode::Mute => NonZeroU8::new(120).unwrap(),
            VirtualKeyCode::MyComputer => NonZeroU8::new(121).unwrap(),
            VirtualKeyCode::NavigateForward => NonZeroU8::new(122).unwrap(),
            VirtualKeyCode::NavigateBackward => NonZeroU8::new(123).unwrap(),
            VirtualKeyCode::NextTrack => NonZeroU8::new(124).unwrap(),
            VirtualKeyCode::NoConvert => NonZeroU8::new(125).unwrap(),
            VirtualKeyCode::NumpadComma => NonZeroU8::new(126).unwrap(),
            VirtualKeyCode::NumpadEnter => NonZeroU8::new(127).unwrap(),
            VirtualKeyCode::NumpadEquals => NonZeroU8::new(128).unwrap(),
            VirtualKeyCode::OEM102 => NonZeroU8::new(129).unwrap(),
            VirtualKeyCode::Period => NonZeroU8::new(130).unwrap(),
            VirtualKeyCode::PlayPause => NonZeroU8::new(131).unwrap(),
            VirtualKeyCode::Power => NonZeroU8::new(132).unwrap(),
            VirtualKeyCode::PrevTrack => NonZeroU8::new(133).unwrap(),
            VirtualKeyCode::RAlt => NonZeroU8::new(134).unwrap(),
            VirtualKeyCode::RBracket => NonZeroU8::new(135).unwrap(),
            VirtualKeyCode::RControl => NonZeroU8::new(136).unwrap(),
            VirtualKeyCode::RShift => NonZeroU8::new(137).unwrap(),
            VirtualKeyCode::RWin => NonZeroU8::new(138).unwrap(),
            VirtualKeyCode::Semicolon => NonZeroU8::new(139).unwrap(),
            VirtualKeyCode::Slash => NonZeroU8::new(140).unwrap(),
            VirtualKeyCode::Sleep => NonZeroU8::new(141).unwrap(),
            VirtualKeyCode::Stop => NonZeroU8::new(142).unwrap(),
            VirtualKeyCode::Subtract => NonZeroU8::new(143).unwrap(),
            VirtualKeyCode::Sysrq => NonZeroU8::new(144).unwrap(),
            VirtualKeyCode::Tab => NonZeroU8::new(145).unwrap(),
            VirtualKeyCode::Underline => NonZeroU8::new(146).unwrap(),
            VirtualKeyCode::Unlabeled => NonZeroU8::new(147).unwrap(),
            VirtualKeyCode::VolumeDown => NonZeroU8::new(148).unwrap(),
            VirtualKeyCode::VolumeUp => NonZeroU8::new(149).unwrap(),
            VirtualKeyCode::Wake => NonZeroU8::new(150).unwrap(),
            VirtualKeyCode::WebBack => NonZeroU8::new(151).unwrap(),
            VirtualKeyCode::WebFavorites => NonZeroU8::new(152).unwrap(),
            VirtualKeyCode::WebForward => NonZeroU8::new(153).unwrap(),
            VirtualKeyCode::WebHome => NonZeroU8::new(154).unwrap(),
            VirtualKeyCode::WebRefresh => NonZeroU8::new(155).unwrap(),
            VirtualKeyCode::WebSearch => NonZeroU8::new(156).unwrap(),
            VirtualKeyCode::WebStop => NonZeroU8::new(157).unwrap(),
            VirtualKeyCode::Yen => NonZeroU8::new(158).unwrap(),
            VirtualKeyCode::Copy => NonZeroU8::new(159).unwrap(),
            VirtualKeyCode::Paste => NonZeroU8::new(160).unwrap(),
            VirtualKeyCode::Cut => NonZeroU8::new(161).unwrap(),
        }
    }
}
