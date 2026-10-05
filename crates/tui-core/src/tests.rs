use crate::*;

fn st() -> Style {
    Style::new(
        Color::Indexed(2),
        Color::Rgb(1, 2, 3),
        Attrs::BOLD.union(Attrs::UNDERLINE),
    )
}

#[test]
fn os_name_is_set() {
    assert_eq!(OS_NAME, "TUI-OS");
}

#[test]
fn attrs_combine() {
    let a = Attrs::BOLD.union(Attrs::REVERSE);
    assert!(a.contains(Attrs::BOLD) && a.contains(Attrs::REVERSE));
    assert!(!a.contains(Attrs::DIM));
    assert!(Attrs::NONE.contains(Attrs::NONE));
}

#[test]
fn new_buffer_is_blank_and_any_size() {
    for (w, h) in [(0, 0), (1, 1), (120, 40), (7, 3)] {
        let b = Buffer::new(w, h);
        assert_eq!((b.width(), b.height()), (w, h));
        if w > 0 && h > 0 {
            assert_eq!(b.get(w - 1, h - 1), Some(&Cell::default()));
        }
    }
}

#[test]
fn set_get_and_out_of_bounds() {
    let mut b = Buffer::new(4, 2);
    let c = Cell::new('x', st());
    assert!(b.set(3, 1, c));
    assert_eq!(b.get(3, 1), Some(&c));
    assert!(!b.set(4, 0, c));
    assert!(!b.set(0, 2, c));
    assert_eq!(b.get(4, 0), None);
}

#[test]
fn put_str_clips_on_the_right() {
    let mut b = Buffer::new(5, 1);
    assert_eq!(b.put_str(3, 0, "hello", st()), 2);
    assert_eq!(b.get(3, 0).unwrap().ch, 'h');
    assert_eq!(b.get(4, 0).unwrap().ch, 'e');
    assert_eq!(b.put_str(9, 0, "x", st()), 0);
    assert_eq!(b.put_str(u16::MAX, 0, "xy", st()), 0);
}

#[test]
fn resize_keeps_overlap_and_blanks_new_area() {
    let mut b = Buffer::new(3, 3);
    b.set(2, 2, Cell::new('z', st()));
    b.set(0, 0, Cell::new('a', st()));
    b.resize(5, 2);
    assert_eq!((b.width(), b.height()), (5, 2));
    assert_eq!(b.get(0, 0).unwrap().ch, 'a');
    assert_eq!(b.get(4, 1), Some(&Cell::default()));
    b.resize(3, 3);
    assert_eq!(
        b.get(2, 2),
        Some(&Cell::default()),
        "lo recortado no vuelve"
    );
}

#[test]
fn blit_clips_with_negative_and_large_offsets() {
    let mut src = Buffer::new(2, 2);
    src.fill(Cell::new('s', st()));
    let mut dst = Buffer::new(3, 3);
    dst.blit(&src, -1, -1);
    assert_eq!(dst.get(0, 0).unwrap().ch, 's');
    assert_eq!(dst.get(1, 0), Some(&Cell::default()));
    dst.blit(&src, 2, 2);
    assert_eq!(dst.get(2, 2).unwrap().ch, 's');
    dst.blit(&src, 100, -100); // totalmente fuera: no entra en pánico
}

#[test]
fn diff_reports_only_changes() {
    let a = Buffer::new(3, 2);
    let mut b = a.clone();
    assert!(b.diff(&a).is_empty());
    let c = Cell::new('q', st());
    b.set(1, 1, c);
    assert_eq!(b.diff(&a), vec![(1, 1, c)]);
}

#[test]
fn diff_after_resize_repaints_everything() {
    let a = Buffer::new(2, 2);
    let b = Buffer::new(3, 2);
    assert_eq!(b.diff(&a).len(), 6);
}

#[test]
fn rect_union() {
    let e = Rect::default();
    let r = Rect::new(2, 3, 4, 1);
    assert_eq!(e.union(r), r);
    assert_eq!(r.union(e), r);
    assert_eq!(
        Rect::new(0, 0, 1, 1).union(Rect::new(4, 5, 2, 2)),
        Rect::new(0, 0, 6, 7)
    );
    let big = Rect::new(u16::MAX - 1, 0, 2, 1);
    assert_eq!(big.union(Rect::new(0, 0, 1, 1)).width, u16::MAX);
}

#[test]
fn surface_tracks_damage() {
    let mut s = Surface::new(10, 4);
    assert_eq!(s.take_damage(), Rect::new(0, 0, 10, 4), "primera vez: todo");
    assert!(s.take_damage().is_empty());
    s.set(2, 1, Cell::new('a', st()));
    s.put_str(5, 3, "bc", st());
    assert_eq!(s.take_damage(), Rect::new(2, 1, 5, 3));
    s.set(99, 99, Cell::default()); // fuera: sin daño
    assert!(s.take_damage().is_empty());
}

#[test]
fn surface_resize_is_a_normal_event() {
    let mut s = Surface::new(4, 4);
    s.take_damage();
    s.resize(6, 2);
    assert_eq!((s.width(), s.height()), (6, 2));
    assert_eq!(s.take_damage(), Rect::new(0, 0, 6, 2));
}
