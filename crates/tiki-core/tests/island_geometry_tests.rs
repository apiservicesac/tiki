use tiki_core::preferences::{IslandGeometry, IslandPlacement, ScreenRect};

/// A 1920×1080 screen with a 48-pixel taskbar at the bottom.
const WORK_AREA: ScreenRect = ScreenRect {
    x: 0,
    y: 0,
    width: 1920,
    height: 1032,
};

#[test]
fn top_and_bottom_center_the_island_horizontally() {
    assert_eq!(
        IslandGeometry::window_origin(IslandPlacement::Top, WORK_AREA, 420, 60),
        (750, 0)
    );
    assert_eq!(
        IslandGeometry::window_origin(IslandPlacement::Bottom, WORK_AREA, 420, 60),
        (750, 972)
    );
}

#[test]
fn left_and_right_center_the_island_vertically() {
    assert_eq!(
        IslandGeometry::window_origin(IslandPlacement::Left, WORK_AREA, 200, 400),
        (0, 316)
    );
    assert_eq!(
        IslandGeometry::window_origin(IslandPlacement::Right, WORK_AREA, 200, 400),
        (1720, 316)
    );
}

#[test]
fn a_second_monitor_keeps_its_offset() {
    let second_monitor = ScreenRect {
        x: 1920,
        y: 0,
        width: 1280,
        height: 1024,
    };
    assert_eq!(
        IslandGeometry::window_origin(IslandPlacement::Top, second_monitor, 280, 40),
        (2420, 0)
    );
}
