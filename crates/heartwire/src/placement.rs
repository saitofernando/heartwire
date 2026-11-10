#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// A monitor and its work area (the monitor minus the taskbar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Screen {
    pub monitor: Bounds,
    pub work: Bounds,
}

const GAP: i32 = 8;

/// Place the panel against the taskbar edge, centred on the tray icon along
/// that edge. The icon can sit in the hidden-icons flyout above the taskbar,
/// so only its position along the edge is used, never its distance from it.
pub fn popup_position(
    anchor: Option<Bounds>,
    width: i32,
    height: i32,
    screen: Screen,
) -> (i32, i32) {
    let Screen { monitor, work } = screen;
    let (cx, cy) = anchor.map_or((work.right, work.bottom), |icon| {
        ((icon.left + icon.right) / 2, (icon.top + icon.bottom) / 2)
    });
    // The taskbar is on the side where the work area is smaller than the
    // monitor; an auto-hidden taskbar falls back to the bottom.
    let (x, y) = if work.top > monitor.top {
        (cx - width / 2, work.top + GAP)
    } else if work.left > monitor.left {
        (work.left + GAP, cy - height / 2)
    } else if work.right < monitor.right {
        (work.right - width - GAP, cy - height / 2)
    } else {
        (cx - width / 2, work.bottom - height - GAP)
    };
    (
        x.clamp(
            work.left + GAP,
            (work.right - width - GAP).max(work.left + GAP),
        ),
        y.clamp(
            work.top + GAP,
            (work.bottom - height - GAP).max(work.top + GAP),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONITOR: Bounds = Bounds {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };
    const BOTTOM: Screen = Screen {
        monitor: MONITOR,
        work: Bounds {
            bottom: 1032,
            ..MONITOR
        },
    };

    fn icon(left: i32, top: i32) -> Option<Bounds> {
        Some(Bounds {
            left,
            top,
            right: left + 24,
            bottom: top + 24,
        })
    }

    #[test]
    fn popup_sits_above_a_bottom_taskbar_icon() {
        assert_eq!(
            popup_position(icon(1700, 1044), 380, 500, BOTTOM),
            (1522, 524)
        );
    }

    #[test]
    fn hidden_icons_flyout_still_anchors_to_the_taskbar() {
        // The flyout floats above the taskbar, inside the work area.
        assert_eq!(
            popup_position(icon(1650, 900), 380, 500, BOTTOM),
            (1472, 524)
        );
    }

    #[test]
    fn popup_stays_inside_the_screen() {
        assert_eq!(
            popup_position(icon(1896, 1044), 380, 500, BOTTOM),
            (1532, 524)
        );
    }

    #[test]
    fn popup_follows_top_and_side_taskbars() {
        let top = Screen {
            monitor: MONITOR,
            work: Bounds { top: 48, ..MONITOR },
        };
        assert_eq!(popup_position(icon(1700, 12), 380, 500, top), (1522, 56));
        let left = Screen {
            monitor: MONITOR,
            work: Bounds {
                left: 48,
                ..MONITOR
            },
        };
        assert_eq!(popup_position(icon(12, 900), 380, 500, left), (56, 572));
        let right = Screen {
            monitor: MONITOR,
            work: Bounds {
                right: 1872,
                ..MONITOR
            },
        };
        assert_eq!(
            popup_position(icon(1884, 900), 380, 500, right),
            (1484, 572)
        );
    }

    #[test]
    fn popup_without_icon_uses_bottom_right() {
        assert_eq!(popup_position(None, 380, 500, BOTTOM), (1532, 524));
    }
}
