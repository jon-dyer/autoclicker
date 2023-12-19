use std::fmt::Display;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum ClickToggle {
    Left,
    Right,
    Both,
    #[default]
    Neither,
}

impl ClickToggle {
    pub fn toggle_left(&self) -> ClickToggle {
        match self {
            Self::Left => Self::Neither,
            Self::Right => Self::Both,
            Self::Both => Self::Right,
            Self::Neither => Self::Left,
        }
    }

    pub fn toggle_right(&self) -> ClickToggle {
        match self {
            Self::Left => Self::Both,
            Self::Right => Self::Neither,
            Self::Both => Self::Left,
            Self::Neither => Self::Right,
        }
    }

    pub fn not_left(&self) -> bool {
        match self {
            Self::Left | Self::Both => false,
            Self::Right | Self::Neither => true,
        }
    }

    pub fn not_right(&self) -> bool {
        match self {
            Self::Left | Self::Neither => true,
            Self::Right | Self::Both => false,
        }
    }

    pub fn set_left(&self, pressed: bool) -> ClickToggle {
        match pressed {
            true => self.press_left(),
            false => self.unpress_left(),
        }
    }

    pub fn set_right(&self, pressed: bool) -> ClickToggle {
        match pressed {
            true => self.press_right(),
            false => self.unpress_right(),
        }
    }

    fn press_right(&self) -> ClickToggle {
        match self {
            Self::Left => Self::Both,
            Self::Right | Self::Neither => Self::Right,
            Self::Both => Self::Both,
        }
    }

    fn unpress_right(&self) -> ClickToggle {
        match self {
            Self::Right => Self::Neither,
            Self::Both => Self::Left,
            Self::Left | Self::Neither => self.clone(),
        }
    }

    fn press_left(&self) -> ClickToggle {
        match self {
            Self::Right => ClickToggle::Both,
            Self::Neither => ClickToggle::Left,
            Self::Left | Self::Both => self.clone(),
        }
    }

    fn unpress_left(&self) -> ClickToggle {
        match self {
            Self::Left => Self::Neither,
            Self::Both => Self::Right,
            Self::Right | Self::Neither => self.clone(),
        }
    }
}

impl Display for ClickToggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ClickToggle::Both => "left, right",
                ClickToggle::Left => "left",
                ClickToggle::Right => "right",
                ClickToggle::Neither => "",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use super::*;

    #[bench]
    fn print_idk(b: &mut Bencher) {
        b.iter(|| {
            let toggle: ClickToggle = ClickToggle::Left;
            toggle.to_string();
        });
    }

    mod toggle_states {
        use super::super::*;

        #[test]
        fn can_toggle_left() {
            let mut subject = ClickToggle::default();
            assert_eq!(ClickToggle::Neither, subject, "starts false");
            subject = subject.toggle_left();
            assert_eq!(ClickToggle::Left, subject, "toggles fore");
            subject = subject.toggle_left();
            assert_eq!(ClickToggle::Neither, subject, "and back");
        }

        #[test]
        fn can_toggle_right() {
            let mut subject = ClickToggle::default();
            assert_eq!(ClickToggle::Neither, subject, "starts false");
            subject = subject.toggle_right();
            assert_eq!(ClickToggle::Right, subject, "toggles fore");
            subject = subject.toggle_right();
            assert_eq!(ClickToggle::Neither, subject, "starts false");
        }

        #[test]
        fn can_toggle_correctly_both() {
            assert_eq!(ClickToggle::Both, ClickToggle::Right.toggle_left(),);
            assert_eq!(ClickToggle::Both, ClickToggle::Left.toggle_right(),);
            assert_eq!(ClickToggle::Right, ClickToggle::Both.toggle_left(),);
            assert_eq!(ClickToggle::Left, ClickToggle::Both.toggle_right(),);
        }

        #[test]
        fn both_is_left() {
            assert!(!ClickToggle::Both.not_left());
        }

        #[test]
        fn both_is_right() {
            assert!(!ClickToggle::Both.not_right());
        }

        #[test]
        fn neither_is_not_left() {
            assert!(ClickToggle::Neither.not_left());
        }

        #[test]
        fn neither_is_not_right() {
            assert!(ClickToggle::Neither.not_right());
        }

        #[test]
        fn right_is_not_left() {
            assert!(ClickToggle::Right.not_left());
        }

        #[test]
        fn left_is_not_right() {
            assert!(ClickToggle::Left.not_right());
        }

        #[test]
        fn set_lefts_will_change() {
            assert_eq!(ClickToggle::Left, ClickToggle::Neither.set_left(true));
            assert_eq!(ClickToggle::Both, ClickToggle::Right.set_left(true));
            assert_eq!(ClickToggle::Neither, ClickToggle::Left.set_left(false));
            assert_eq!(ClickToggle::Right, ClickToggle::Both.set_left(false));
        }

        #[test]
        fn set_lefts_will_remain() {
            assert_eq!(ClickToggle::Neither, ClickToggle::Neither.set_left(false));
            assert_eq!(ClickToggle::Right, ClickToggle::Right.set_left(false));
            assert_eq!(ClickToggle::Both, ClickToggle::Both.set_left(true));
            assert_eq!(ClickToggle::Left, ClickToggle::Left.set_left(true));
        }

        #[test]
        fn set_rights_will_change() {
            assert_eq!(ClickToggle::Right, ClickToggle::Neither.set_right(true));
            assert_eq!(ClickToggle::Both, ClickToggle::Left.set_right(true));
            assert_eq!(ClickToggle::Neither, ClickToggle::Right.set_right(false));
            assert_eq!(ClickToggle::Left, ClickToggle::Both.set_right(false));
        }

        #[test]
        fn set_rights_will_remain() {
            assert_eq!(ClickToggle::Neither, ClickToggle::Neither.set_right(false));
            assert_eq!(ClickToggle::Left, ClickToggle::Left.set_right(false));
            assert_eq!(ClickToggle::Both, ClickToggle::Both.set_right(true));
            assert_eq!(ClickToggle::Right, ClickToggle::Right.set_right(true));
        }
    }

    mod toggle_states_string {
        use super::*;

        #[test]
        fn left_is_left() {
            let toggle: ClickToggle = ClickToggle::Left;
            assert_eq!("left", &toggle.to_string());
        }

        #[test]
        fn right_is_right() {
            let toggle: ClickToggle = ClickToggle::Right;
            assert_eq!("right", &toggle.to_string());
        }

        #[test]
        fn both_are_comma_separated() {
            let toggle: ClickToggle = ClickToggle::Both;
            assert_eq!("left, right", &toggle.to_string());
        }

        #[test]
        fn none_is_blank() {
            let toggle: ClickToggle = ClickToggle::Neither;
            assert_eq!("", &toggle.to_string());
        }
    }
}
