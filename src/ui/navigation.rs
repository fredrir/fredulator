use gtk::prelude::*;

use crate::ui::keyboard::Direction;

pub struct NavButton {
    pub button: gtk::Button,
    pub col: usize,
    pub row: usize,
    pub scientific: bool,
}

pub fn navigate(nav: &[NavButton], dir: Direction, scientific: bool) {
    let visible: Vec<&NavButton> = nav.iter().filter(|b| !b.scientific || scientific).collect();
    let current = visible.iter().find(|b| b.button.has_focus());
    if let Some(cur) = current {
        let (cc, cr) = eff_pos(cur, scientific);
        let target = match dir {
            Direction::Left => visible
                .iter()
                .filter(|b| eff_pos(b, scientific).1 == cr && eff_pos(b, scientific).0 < cc)
                .max_by_key(|b| eff_pos(b, scientific).0),
            Direction::Right => visible
                .iter()
                .filter(|b| eff_pos(b, scientific).1 == cr && eff_pos(b, scientific).0 > cc)
                .min_by_key(|b| eff_pos(b, scientific).0),
            Direction::Up => visible
                .iter()
                .filter(|b| eff_pos(b, scientific).0 == cc && eff_pos(b, scientific).1 < cr)
                .max_by_key(|b| eff_pos(b, scientific).1),
            Direction::Down => visible
                .iter()
                .filter(|b| eff_pos(b, scientific).0 == cc && eff_pos(b, scientific).1 > cr)
                .min_by_key(|b| eff_pos(b, scientific).1),
        };
        if let Some(t) = target {
            t.button.grab_focus();
        }
    } else if let Some(first) = visible.first() {
        first.button.grab_focus();
    }
}

pub fn activate_focused(nav: &[NavButton], scientific: bool) {
    let visible: Vec<&NavButton> = nav.iter().filter(|b| !b.scientific || scientific).collect();
    if let Some(b) = visible.iter().find(|b| b.button.has_focus()) {
        b.button.clicked();
    }
}

fn eff_pos(b: &NavButton, scientific: bool) -> (usize, usize) {
    if b.scientific {
        (b.col, b.row)
    } else if scientific {
        (b.col + 3, b.row)
    } else {
        (b.col, b.row)
    }
}
