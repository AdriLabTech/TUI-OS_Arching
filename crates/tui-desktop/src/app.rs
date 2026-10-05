//! Estado y comportamiento del escritorio. La entrada llega como `Event` de `tui-core`
//! y el dibujo va a un `Buffer`: nada aquí conoce el terminal real.

use std::path::PathBuf;

use tui_core::{
    Buffer, Event, KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseKind, Rect,
};

use crate::sys::{self, CpuTimes, Entry, MemInfo, Proc, Snapshot};
use crate::theme::{self, style};
use crate::ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Home,
    System,
    Processes,
    Files,
    Help,
}

pub const SECTIONS: [(Section, &str); 5] = [
    (Section::Home, "INICIO"),
    (Section::System, "SISTEMA"),
    (Section::Processes, "PROCESOS"),
    (Section::Files, "ARCHIVOS"),
    (Section::Help, "AYUDA"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Rail,
    Content,
}

/// Rectángulos de la pantalla; todo se deriva del tamaño que da el terminal.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub top: Rect,
    pub rail: Rect,
    pub content: Rect,
    pub side: Option<Rect>,
    pub bottom: Rect,
}

pub const MIN_SIZE: (u16, u16) = (48, 12);

pub fn layout(w: u16, h: u16) -> Layout {
    let rail_w = if w >= 72 { 20 } else { 14 };
    let side_w = if w >= 110 { 30 } else { 0 };
    let mid_h = h.saturating_sub(2);
    Layout {
        top: Rect::new(0, 0, w, 1),
        rail: Rect::new(0, 1, rail_w.min(w), mid_h),
        content: Rect::new(rail_w, 1, w.saturating_sub(rail_w + side_w), mid_h),
        side: (side_w > 0).then(|| Rect::new(w - side_w, 1, side_w, mid_h)),
        bottom: Rect::new(0, h.saturating_sub(1), w, 1),
    }
}

struct Files {
    cwd: PathBuf,
    entries: Vec<Entry>,
    sel: usize,
    err: Option<String>,
}

impl Files {
    fn open(path: PathBuf) -> Files {
        let mut f = Files {
            cwd: path,
            entries: Vec::new(),
            sel: 0,
            err: None,
        };
        f.reload();
        f
    }

    fn reload(&mut self) {
        match sys::read_dir(&self.cwd) {
            Ok(e) => (self.entries, self.err) = (e, None),
            Err(e) => (self.entries, self.err) = (Vec::new(), Some(e.to_string())),
        }
        self.sel = 0;
    }

    fn enter(&mut self) {
        let Some(e) = self.entries.get(self.sel) else {
            return;
        };
        if e.is_dir {
            let next = self.cwd.join(&e.name);
            let prev = std::mem::replace(&mut self.cwd, next);
            self.reload();
            if self.err.is_some() && self.entries.is_empty() {
                // sin permiso: mostrar el error pero poder volver
                let msg = self.err.take();
                self.cwd = prev;
                self.reload();
                self.err = msg;
            }
        }
    }

    fn up(&mut self) {
        if let Some(parent) = self.cwd.parent().map(|p| p.to_path_buf()) {
            let from = self
                .cwd
                .file_name()
                .map(|n| n.to_string_lossy().into_owned());
            self.cwd = parent;
            self.reload();
            if let Some(from) = from {
                self.sel = self
                    .entries
                    .iter()
                    .position(|e| e.name == from)
                    .unwrap_or(0);
            }
        }
    }
}

pub struct App {
    pub section: Section,
    pub focus: Focus,
    pub quit: bool,
    size: (u16, u16),
    host: String,
    kernel: String,
    os: String,
    cpu_model: String,
    snap: Snapshot,
    prev_cpu: Vec<CpuTimes>,
    cpu_hist: Vec<f32>,
    core_use: Vec<f32>,
    procs: Vec<Proc>,
    proc_sel: usize,
    files: Files,
    ticks: u64,
}

const HIST: usize = 240;

impl App {
    pub fn new(size: (u16, u16)) -> App {
        let snap = sys::snapshot();
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .unwrap_or_else(|| "/".into());
        App {
            section: Section::Home,
            focus: Focus::Rail,
            quit: false,
            size,
            host: sys::hostname(),
            kernel: sys::kernel_release(),
            os: sys::os_name(),
            cpu_model: sys::cpu_model(),
            prev_cpu: snap.cpu.clone(),
            snap,
            cpu_hist: Vec::new(),
            core_use: Vec::new(),
            procs: sys::processes(),
            proc_sel: 0,
            files: Files::open(home),
            ticks: 0,
        }
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.size = (w, h);
    }

    /// Muestreo periódico (≈1 s): CPU, memoria, procesos.
    pub fn tick(&mut self) {
        self.ticks += 1;
        let snap = sys::snapshot();
        let usage: Vec<f32> = snap
            .cpu
            .iter()
            .zip(&self.prev_cpu)
            .map(|(n, p)| n.usage_since(p))
            .collect();
        if let Some((total, cores)) = usage.split_first() {
            self.cpu_hist.push(*total);
            if self.cpu_hist.len() > HIST {
                self.cpu_hist.remove(0);
            }
            self.core_use = cores.to_vec();
        }
        self.prev_cpu = snap.cpu.clone();
        self.snap = snap;
        if self.section == Section::Processes || self.ticks.is_multiple_of(3) {
            self.procs = sys::processes();
            self.proc_sel = self.proc_sel.min(self.procs.len().saturating_sub(1));
        }
    }

    fn cpu_now(&self) -> f32 {
        self.cpu_hist.last().copied().unwrap_or(0.0)
    }

    // ---------------------------------------------------------------- entrada

    pub fn on_event(&mut self, ev: Event) {
        match ev {
            Event::Key(k) => self.on_key(k),
            Event::Mouse(m) => self.on_mouse(m),
            Event::Resize { width, height } => self.resize(width, height),
        }
    }

    fn select(&mut self, i: usize) {
        if let Some((s, _)) = SECTIONS.get(i) {
            self.section = *s;
        }
    }

    fn section_index(&self) -> usize {
        SECTIONS
            .iter()
            .position(|(s, _)| *s == self.section)
            .unwrap_or(0)
    }

    fn on_key(&mut self, k: KeyEvent) {
        if k.is(KeyCode::Char('q'), Modifiers::CTRL) || k.is(KeyCode::Char('q'), Modifiers::NONE) {
            self.quit = true;
            return;
        }
        if k.is(KeyCode::F(1), Modifiers::NONE) {
            self.section = Section::Help;
            return;
        }
        if let KeyCode::Char(c @ '0'..='4') = k.code
            && k.mods == Modifiers::NONE
        {
            self.select(c as usize - '0' as usize);
            return;
        }
        match self.focus {
            Focus::Rail => match k.code {
                KeyCode::Up => self.select(self.section_index().saturating_sub(1)),
                KeyCode::Down => self.select((self.section_index() + 1).min(SECTIONS.len() - 1)),
                KeyCode::Enter | KeyCode::Right | KeyCode::Tab => self.focus = Focus::Content,
                _ => {}
            },
            Focus::Content => {
                if matches!(k.code, KeyCode::Esc | KeyCode::Tab | KeyCode::BackTab) {
                    self.focus = Focus::Rail;
                    return;
                }
                self.content_key(k.code);
            }
        }
    }

    fn visible_rows(&self) -> usize {
        // marco (2) + cabecera (1) en las listas
        layout(self.size.0, self.size.1)
            .content
            .height
            .saturating_sub(3)
            .max(1) as usize
    }

    fn content_key(&mut self, code: KeyCode) {
        let page = self.visible_rows();
        match self.section {
            Section::Processes => {
                let max = self.procs.len().saturating_sub(1);
                self.proc_sel = match code {
                    KeyCode::Up => self.proc_sel.saturating_sub(1),
                    KeyCode::Down => (self.proc_sel + 1).min(max),
                    KeyCode::PageUp => self.proc_sel.saturating_sub(page),
                    KeyCode::PageDown => (self.proc_sel + page).min(max),
                    KeyCode::Home => 0,
                    KeyCode::End => max,
                    _ => self.proc_sel,
                };
            }
            Section::Files => {
                let max = self.files.entries.len().saturating_sub(1);
                match code {
                    KeyCode::Up => self.files.sel = self.files.sel.saturating_sub(1),
                    KeyCode::Down => self.files.sel = (self.files.sel + 1).min(max),
                    KeyCode::PageUp => self.files.sel = self.files.sel.saturating_sub(page),
                    KeyCode::PageDown => self.files.sel = (self.files.sel + page).min(max),
                    KeyCode::Home => self.files.sel = 0,
                    KeyCode::End => self.files.sel = max,
                    KeyCode::Enter | KeyCode::Right => self.files.enter(),
                    KeyCode::Backspace | KeyCode::Left => self.files.up(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn on_mouse(&mut self, m: MouseEvent) {
        let l = layout(self.size.0, self.size.1);
        let inside =
            |r: Rect| m.x >= r.x && m.x < r.x + r.width && m.y >= r.y && m.y < r.y + r.height;
        match m.kind {
            MouseKind::Down(MouseButton::Left) if inside(l.rail) => {
                self.focus = Focus::Rail;
                for i in 0..SECTIONS.len() {
                    if m.y == rail_item_y(l.rail, i) {
                        self.select(i);
                    }
                }
            }
            MouseKind::Down(MouseButton::Left) if inside(l.content) => {
                self.focus = Focus::Content;
                // fila de lista: contenido + marco + cabecera
                let first = l.content.y + 2;
                if m.y >= first {
                    let row = (m.y - first) as usize;
                    match self.section {
                        Section::Processes => {
                            let i = self.list_offset(self.proc_sel) + row;
                            if i < self.procs.len() {
                                self.proc_sel = i;
                            }
                        }
                        Section::Files => {
                            let i = self.list_offset(self.files.sel) + row;
                            if i < self.files.entries.len() {
                                if self.files.sel == i {
                                    self.files.enter();
                                } else {
                                    self.files.sel = i;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            MouseKind::ScrollUp | MouseKind::ScrollDown if inside(l.content) => {
                let code = if m.kind == MouseKind::ScrollUp {
                    KeyCode::Up
                } else {
                    KeyCode::Down
                };
                for _ in 0..3 {
                    self.content_key(code);
                }
            }
            _ => {}
        }
    }

    /// Primera fila visible de una lista para mantener visible el elemento seleccionado.
    fn list_offset(&self, sel: usize) -> usize {
        sel.saturating_sub(self.visible_rows().saturating_sub(1))
    }

    // ----------------------------------------------------------------- dibujo

    pub fn draw(&self, b: &mut Buffer) {
        let (w, h) = (b.width(), b.height());
        ui::fill(b, Rect::new(0, 0, w, h), theme::PAPER);
        if w < MIN_SIZE.0 || h < MIN_SIZE.1 {
            let msg = format!(
                "Terminal demasiado pequeño ({w}x{h}); mínimo {}x{}",
                MIN_SIZE.0, MIN_SIZE.1
            );
            ui::text(b, 0, h / 2, w, &msg, style(theme::INK, theme::PAPER));
            return;
        }
        let l = layout(w, h);
        self.draw_top(b, l.top);
        self.draw_rail(b, l.rail);
        self.draw_content(b, l.content);
        if let Some(side) = l.side {
            self.draw_side(b, side);
        }
        self.draw_bottom(b, l.bottom);
    }

    fn draw_top(&self, b: &mut Buffer, r: Rect) {
        ui::fill(b, r, theme::RAISED);
        let brand = ui::text(
            b,
            0,
            r.y,
            r.width,
            " TUI-OS ",
            style(theme::BRIGHT, theme::VERMILION),
        );
        ui::text(
            b,
            brand + 1,
            r.y,
            r.width,
            &self.host,
            style(theme::INK_DIM, theme::RAISED),
        );
        let status = format!(
            "CPU {:>3.0}%  MEM {:>3.0}%  {} UTC ",
            self.cpu_now() * 100.0,
            self.snap.mem.used_frac() * 100.0,
            sys::clock_utc()
        );
        let n = status.chars().count() as u16;
        if r.width > brand + n + 12 {
            ui::text(
                b,
                r.width - n,
                r.y,
                n,
                &status,
                style(theme::INK, theme::RAISED),
            );
        }
    }

    fn draw_bottom(&self, b: &mut Buffer, r: Rect) {
        ui::fill(b, r, theme::RAISED);
        let hints = [
            ("F1", "Ayuda"),
            ("↑↓", "Mover"),
            ("ENTER", "Abrir"),
            ("TAB", "Foco"),
            ("ESC", "Atrás"),
            ("CTRL+Q", "Salir"),
        ];
        let mut x = 1;
        for (key, label) in hints {
            let used = ui::text(
                b,
                x,
                r.y,
                r.width.saturating_sub(x),
                key,
                style(theme::BRIGHT, theme::HIGH),
            );
            let used2 = ui::text(
                b,
                x + used + 1,
                r.y,
                r.width.saturating_sub(x + used + 1),
                label,
                style(theme::MUTED, theme::RAISED),
            );
            x += used + used2 + 3;
            if x >= r.width {
                break;
            }
        }
    }

    fn draw_rail(&self, b: &mut Buffer, r: Rect) {
        ui::fill(b, r, theme::RAISED);
        ui::text(
            b,
            r.x + 2,
            r.y,
            r.width.saturating_sub(2),
            "MENÚ",
            style(theme::FAINT, theme::RAISED),
        );
        for (i, (s, name)) in SECTIONS.iter().enumerate() {
            let y = rail_item_y(r, i);
            if y >= r.y + r.height {
                break;
            }
            let selected = *s == self.section;
            let bg = if selected { theme::HIGH } else { theme::RAISED };
            ui::fill(b, Rect::new(r.x, y, r.width, 1), bg);
            if selected {
                let marker = if self.focus == Focus::Rail {
                    theme::VERMILION
                } else {
                    theme::LINE
                };
                ui::text(b, r.x, y, 1, "▌", style(marker, bg));
            }
            ui::text(b, r.x + 2, y, 2, &format!("{i:02}"), style(theme::GOLD, bg));
            let fg = if selected {
                theme::BRIGHT
            } else {
                theme::INK_DIM
            };
            ui::text(
                b,
                r.x + 5,
                y,
                r.width.saturating_sub(6),
                name,
                style(fg, bg),
            );
        }
        let foot = r.y + r.height.saturating_sub(1);
        ui::text(
            b,
            r.x + 2,
            foot,
            r.width.saturating_sub(2),
            "v0.0.1",
            style(theme::FAINT, theme::RAISED),
        );
    }

    fn title(&self) -> String {
        let i = self.section_index();
        format!("{i:02} // {}", SECTIONS[i].1)
    }

    fn draw_content(&self, b: &mut Buffer, r: Rect) {
        ui::frame(
            b,
            r,
            &self.title(),
            self.focus == Focus::Content,
            theme::PAPER,
        );
        let inner = ui::inset(r, 1);
        match self.section {
            Section::Home => self.draw_home(b, inner),
            Section::System => self.draw_system(b, inner),
            Section::Processes => self.draw_processes(b, inner),
            Section::Files => self.draw_files(b, inner),
            Section::Help => draw_help(b, inner),
        }
    }

    fn draw_home(&self, b: &mut Buffer, r: Rect) {
        // Disco-sol plano a la derecha: una sola superficie tonal, sin trama.
        let ry = (r.height / 2).saturating_sub(1).min(r.width / 6);
        if ry >= 2 {
            let (cx, cy) = (
                (r.x + r.width).saturating_sub(ry * 2 + 3),
                r.y + r.height / 2,
            );
            for dy in -(ry as i32)..=(ry as i32) {
                for dx in -(2 * ry as i32)..=(2 * ry as i32) {
                    let (fx, fy) = (dx as f32 / 2.0, dy as f32);
                    if fx * fx + fy * fy <= (ry * ry) as f32 {
                        let (x, y) = (cx as i32 + dx, cy as i32 + dy);
                        if x >= r.x as i32 && y >= r.y as i32 {
                            b.set(
                                x as u16,
                                y as u16,
                                tui_core::Cell::new(' ', style(theme::INK, theme::CONTAINER)),
                            );
                        }
                    }
                }
            }
        }
        let logo_w = draw_logo(b, r.x + 2, r.y + 1, r.width.saturating_sub(2));
        let y = r.y + 7;
        ui::text(
            b,
            r.x + 2,
            y,
            r.width,
            "Linux debajo. TUI encima.",
            style(theme::INK_DIM, theme::PAPER),
        );
        let w = logo_w.max(34).min(r.width.saturating_sub(2));
        let rows = [
            ("SISTEMA", self.os.clone()),
            ("KERNEL", self.kernel.clone()),
            ("EQUIPO", self.host.clone()),
            (
                "CPU",
                format!(
                    "{} núcleos · {:.0}%",
                    self.core_use.len().max(1),
                    self.cpu_now() * 100.0
                ),
            ),
            (
                "MEMORIA",
                format!(
                    "{} / {}",
                    sys::human_kb(self.snap.mem.used_kb()),
                    sys::human_kb(self.snap.mem.total_kb)
                ),
            ),
            ("ACTIVO", sys::fmt_uptime(self.snap.uptime_s)),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            ui::leader(b, r.x + 2, y + 2 + i as u16, w, k, v, theme::PAPER);
        }
        if r.height >= 24 && r.width >= 60 {
            self.draw_cards(b, Rect::new(r.x + 1, r.y + r.height - 7, r.width - 2, 7));
        } else if r.height > 16 {
            let foot = "PAPER #000000 · INK #cdc4ba · NO SHADOW";
            ui::text(
                b,
                r.x + 2,
                r.y + r.height - 1,
                r.width.saturating_sub(2),
                foot,
                style(theme::FAINT, theme::PAPER),
            );
        }
    }

    /// Cuatro tarjetas de acceso rápido con un dato vivo de cada sección.
    fn draw_cards(&self, b: &mut Buffer, r: Rect) {
        for (i, (_, name)) in SECTIONS.iter().skip(1).enumerate() {
            let (x0, x1) = (
                r.x + i as u16 * r.width / 4,
                r.x + (i as u16 + 1) * r.width / 4,
            );
            let card = Rect::new(
                x0,
                r.y,
                (x1 - x0).saturating_sub(u16::from(i < 3)),
                r.height,
            );
            let title = if r.width / 4 >= 18 {
                format!("{:02} {name}", i + 1)
            } else {
                name.to_string()
            };
            ui::frame(b, card, &title, false, theme::PAPER);
            let (x, w) = (card.x + 2, card.width.saturating_sub(3));
            let ink = style(theme::INK, theme::PAPER);
            let dim = style(theme::MUTED, theme::PAPER);
            match i {
                0 => {
                    ui::area(
                        b,
                        Rect::new(x, card.y + 1, w, 3),
                        &self.cpu_hist,
                        theme::INK,
                        theme::PAPER,
                    );
                    let cpu = format!("CPU {:.0}%", self.cpu_now() * 100.0);
                    let mem = format!("MEM {:.0}%", self.snap.mem.used_frac() * 100.0);
                    ui::text(b, x, card.y + 4, w, &cpu, dim);
                    ui::text(b, x, card.y + 5, w, &mem, dim);
                }
                1 => {
                    ui::text(
                        b,
                        x,
                        card.y + 1,
                        w,
                        &format!("{} procesos", self.procs.len()),
                        ink,
                    );
                    if let Some(p) = self.procs.first() {
                        ui::text(b, x, card.y + 3, w, &p.name, dim);
                        ui::text(b, x, card.y + 4, w, &sys::human_kb(p.rss_kb), dim);
                    }
                }
                2 => {
                    ui::text(
                        b,
                        x,
                        card.y + 1,
                        w,
                        &format!("{} elementos", self.files.entries.len()),
                        ink,
                    );
                    let path = self.files.cwd.display().to_string();
                    let tail: String = path
                        .chars()
                        .rev()
                        .take(w as usize)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    ui::text(b, x, card.y + 3, w, &tail, dim);
                }
                _ => {
                    ui::text(b, x, card.y + 1, w, "F1  atajos", ink);
                    ui::text(b, x, card.y + 3, w, "Q   salir", dim);
                    ui::text(b, x, card.y + 4, w, "0-4 secciones", dim);
                }
            }
        }
    }

    fn draw_system(&self, b: &mut Buffer, r: Rect) {
        let w = r.width.saturating_sub(4);
        let x = r.x + 2;
        let rows = [
            ("EQUIPO", self.host.clone()),
            ("SISTEMA", self.os.clone()),
            ("KERNEL", self.kernel.clone()),
            ("CPU", self.cpu_model.clone()),
            ("ACTIVO", sys::fmt_uptime(self.snap.uptime_s)),
            (
                "CARGA",
                format!(
                    "{:.2}  {:.2}  {:.2}",
                    self.snap.load[0], self.snap.load[1], self.snap.load[2]
                ),
            ),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            ui::leader(b, x, r.y + 1 + i as u16, w, k, v, theme::PAPER);
        }
        let mut y = r.y + 8;
        let head = style(theme::MUTED, theme::PAPER);
        ui::text(
            b,
            x,
            y,
            w,
            &format!("CPU  {:>3.0}%", self.cpu_now() * 100.0),
            head,
        );
        y += 1;
        let ch = (r.height.saturating_sub(18) / 2).clamp(3, 8);
        if y + ch < r.y + r.height {
            ui::area(
                b,
                Rect::new(x, y, w, ch),
                &self.cpu_hist,
                theme::INK,
                theme::PAPER,
            );
            y += ch + 1;
        }
        // núcleos en dos columnas
        let colw = w / 2;
        for (i, u) in self.core_use.iter().enumerate() {
            let (cx, cy) = (x + (i as u16 % 2) * colw, y + i as u16 / 2);
            if cy >= r.y + r.height.saturating_sub(4) || colw < 12 {
                break;
            }
            ui::text(b, cx, cy, 4, &format!("C{i:<2}"), head);
            ui::hbar(
                b,
                cx + 4,
                cy,
                colw.saturating_sub(10).max(4),
                *u,
                bar_color(*u),
                theme::PAPER,
            );
            ui::text(
                b,
                cx + colw.saturating_sub(5),
                cy,
                4,
                &format!("{:>3.0}%", u * 100.0),
                style(theme::INK, theme::PAPER),
            );
        }
        let my = r.y + r.height.saturating_sub(3);
        mem_bars(b, x, my, w, &self.snap.mem);
    }

    fn draw_processes(&self, b: &mut Buffer, r: Rect) {
        let head = style(theme::MUTED, theme::PAPER);
        ui::text(
            b,
            r.x + 1,
            r.y,
            r.width,
            &format!(
                "{:>7}  {:<3} {:>9}  NOMBRE        ({} procesos)",
                "PID",
                "EST",
                "MEM",
                self.procs.len()
            ),
            head,
        );
        let rows = self.visible_rows().min(r.height.saturating_sub(1) as usize);
        let off = self.list_offset(self.proc_sel);
        for (i, p) in self.procs.iter().skip(off).take(rows).enumerate() {
            let y = r.y + 1 + i as u16;
            let sel = off + i == self.proc_sel;
            let bg = if sel { theme::HIGH } else { theme::PAPER };
            ui::fill(b, Rect::new(r.x, y, r.width, 1), bg);
            let fg = if sel { theme::BRIGHT } else { theme::INK };
            let line = format!(
                "{:>7}  {:<3} {:>9}  {}",
                p.pid,
                p.state,
                sys::human_kb(p.rss_kb),
                p.name
            );
            ui::text(
                b,
                r.x + 1,
                y,
                r.width.saturating_sub(1),
                &line,
                style(fg, bg),
            );
        }
    }

    fn draw_files(&self, b: &mut Buffer, r: Rect) {
        let f = &self.files;
        let path = f.cwd.display().to_string();
        ui::text(
            b,
            r.x + 1,
            r.y,
            r.width.saturating_sub(1),
            &path,
            style(theme::GOLD, theme::PAPER),
        );
        if let Some(e) = &f.err {
            ui::text(
                b,
                r.x + 1,
                r.y + 1,
                r.width.saturating_sub(1),
                e,
                style(theme::ERROR, theme::PAPER),
            );
            return;
        }
        let rows = self.visible_rows().min(r.height.saturating_sub(1) as usize);
        let off = self.list_offset(f.sel);
        for (i, e) in f.entries.iter().skip(off).take(rows).enumerate() {
            let y = r.y + 1 + i as u16;
            let sel = off + i == f.sel;
            let bg = if sel { theme::HIGH } else { theme::PAPER };
            ui::fill(b, Rect::new(r.x, y, r.width, 1), bg);
            let fg = match (sel, e.is_dir) {
                (true, _) => theme::BRIGHT,
                (false, true) => theme::INK,
                (false, false) => theme::INK_DIM,
            };
            let (mark, size) = if e.is_dir {
                ("▸", String::new())
            } else {
                (" ", sys::human(e.size))
            };
            ui::text(
                b,
                r.x + 1,
                y,
                r.width.saturating_sub(1),
                &format!("{mark} {}{}", e.name, if e.is_dir { "/" } else { "" }),
                style(fg, bg),
            );
            if !size.is_empty() && r.width > 30 {
                ui::text(
                    b,
                    r.x + r.width - 9,
                    y,
                    8,
                    &format!("{size:>8}"),
                    style(theme::MUTED, bg),
                );
            }
        }
        if f.entries.is_empty() {
            ui::text(
                b,
                r.x + 1,
                r.y + 1,
                r.width,
                "(vacío)",
                style(theme::FAINT, theme::PAPER),
            );
        }
    }

    fn draw_side(&self, b: &mut Buffer, r: Rect) {
        ui::frame(b, r, "RESUMEN", false, theme::PAPER);
        let r = ui::inset(r, 1);
        let (x, w) = (r.x + 1, r.width.saturating_sub(2));
        let head = style(theme::MUTED, theme::PAPER);
        ui::text(
            b,
            x,
            r.y,
            w,
            &format!("CPU  {:>3.0}%", self.cpu_now() * 100.0),
            head,
        );
        ui::area(
            b,
            Rect::new(x, r.y + 1, w, 4),
            &self.cpu_hist,
            theme::INK,
            theme::PAPER,
        );
        mem_bars(b, x, r.y + 6, w, &self.snap.mem);
        ui::leader(
            b,
            x,
            r.y + 10,
            w,
            "CARGA",
            &format!("{:.2}", self.snap.load[0]),
            theme::PAPER,
        );
        ui::leader(
            b,
            x,
            r.y + 11,
            w,
            "PROCESOS",
            &self.procs.len().to_string(),
            theme::PAPER,
        );
        ui::leader(
            b,
            x,
            r.y + 12,
            w,
            "ACTIVO",
            &sys::fmt_uptime(self.snap.uptime_s),
            theme::PAPER,
        );
    }
}

fn rail_item_y(rail: Rect, i: usize) -> u16 {
    rail.y + 2 + i as u16 * 2
}

fn bar_color(frac: f32) -> tui_core::Color {
    if frac > 0.85 {
        theme::ERROR
    } else {
        theme::INK
    }
}

fn mem_bars(b: &mut Buffer, x: u16, y: u16, w: u16, m: &MemInfo) {
    let head = style(theme::MUTED, theme::PAPER);
    let ink = style(theme::INK, theme::PAPER);
    let label = format!(
        "{} / {}",
        sys::human_kb(m.used_kb()),
        sys::human_kb(m.total_kb)
    );
    ui::text(b, x, y, 4, "MEM", head);
    ui::text(
        b,
        x + w.saturating_sub(label.chars().count() as u16),
        y,
        w,
        &label,
        ink,
    );
    ui::hbar(
        b,
        x,
        y + 1,
        w,
        m.used_frac(),
        bar_color(m.used_frac()),
        theme::PAPER,
    );
    let swap = if m.swap_total_kb == 0 {
        "sin swap".to_string()
    } else {
        format!(
            "{} / {}",
            sys::human_kb(m.swap_used_kb()),
            sys::human_kb(m.swap_total_kb)
        )
    };
    ui::text(b, x, y + 2, 5, "SWAP", head);
    ui::text(
        b,
        x + w.saturating_sub(swap.chars().count() as u16),
        y + 2,
        w,
        &swap,
        ink,
    );
}

const GLYPHS: [[&str; 5]; 6] = [
    ["█████", "  █  ", "  █  ", "  █  ", "  █  "], // T
    ["█   █", "█   █", "█   █", "█   █", " ███ "], // U
    ["███", " █ ", " █ ", " █ ", "███"],           // I
    ["     ", "     ", " ███ ", "     ", "     "], // -
    [" ███ ", "█   █", "█   █", "█   █", " ███ "], // O
    [" ████", "█    ", " ███ ", "    █", "████ "], // S
];

/// Dibuja «TUI-OS» en letras de bloque; devuelve el ancho usado.
fn draw_logo(b: &mut Buffer, x: u16, y: u16, max: u16) -> u16 {
    let mut cx = x;
    for g in GLYPHS {
        let gw = g[0].chars().count() as u16;
        if cx + gw > x + max {
            break;
        }
        for (row, line) in g.iter().enumerate() {
            ui::text(
                b,
                cx,
                y + row as u16,
                gw,
                line,
                style(theme::BRIGHT, theme::PAPER),
            );
        }
        cx += gw + 1;
    }
    cx - x
}

fn draw_help(b: &mut Buffer, r: Rect) {
    let keys = [
        ("F1", "Esta ayuda"),
        ("0-4", "Ir a una sección"),
        ("↑ ↓", "Mover selección"),
        ("ENTER / →", "Abrir / entrar en la sección"),
        ("TAB / ESC", "Cambiar foco entre menú y contenido"),
        ("RePág AvPág", "Desplazar listas"),
        ("Backspace / ←", "Subir de carpeta (Archivos)"),
        ("Ratón", "Clic para elegir; rueda para desplazar"),
        ("Q / CTRL+Q", "Salir"),
    ];
    for (i, (k, d)) in keys.iter().enumerate() {
        let y = r.y + 1 + i as u16;
        ui::text(b, r.x + 2, y, 14, k, style(theme::BRIGHT, theme::PAPER));
        ui::text(
            b,
            r.x + 18,
            y,
            r.width.saturating_sub(18),
            d,
            style(theme::INK_DIM, theme::PAPER),
        );
    }
    ui::text(
        b,
        r.x + 2,
        r.y + 12,
        r.width.saturating_sub(2),
        "Recuperación: CTRL+ALT+F2 abre una consola de texto.",
        style(theme::MUTED, theme::PAPER),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui_core::{Event, KeyEvent};

    fn key(c: KeyCode) -> Event {
        Event::Key(KeyEvent::new(c, Modifiers::NONE))
    }

    fn text_of(b: &Buffer) -> String {
        (0..b.height())
            .flat_map(|y| (0..b.width()).map(move |x| (x, y)))
            .map(|(x, y)| b.get(x, y).unwrap().ch)
            .collect()
    }

    #[test]
    fn layout_regions_do_not_overlap_and_fit() {
        for (w, h) in [(48, 12), (80, 24), (120, 40), (160, 50), (300, 80)] {
            let l = layout(w, h);
            let mut rects = vec![l.top, l.rail, l.content, l.bottom];
            rects.extend(l.side);
            for r in &rects {
                assert!(
                    r.x + r.width <= w && r.y + r.height <= h,
                    "{r:?} fuera de {w}x{h}"
                );
            }
            assert_eq!(l.rail.x + l.rail.width, l.content.x);
            assert_eq!(l.side.is_some(), w >= 110);
        }
    }

    #[test]
    fn quit_keys() {
        for ev in [
            key(KeyCode::Char('q')),
            Event::Key(KeyEvent::new(KeyCode::Char('q'), Modifiers::CTRL)),
        ] {
            let mut a = App::new((80, 24));
            a.on_event(ev);
            assert!(a.quit);
        }
    }

    #[test]
    fn rail_navigation_focus_and_help() {
        let mut a = App::new((80, 24));
        a.on_event(key(KeyCode::Down));
        assert_eq!(a.section, Section::System);
        a.on_event(key(KeyCode::Up));
        a.on_event(key(KeyCode::Up));
        assert_eq!(a.section, Section::Home, "no sale por arriba");
        a.on_event(key(KeyCode::Enter));
        assert_eq!(a.focus, Focus::Content);
        a.on_event(key(KeyCode::Esc));
        assert_eq!(a.focus, Focus::Rail);
        a.on_event(key(KeyCode::F(1)));
        assert_eq!(a.section, Section::Help);
        a.on_event(key(KeyCode::Char('3')));
        assert_eq!(a.section, Section::Files);
    }

    #[test]
    fn mouse_click_selects_rail_item_and_focuses_content() {
        let mut a = App::new((100, 30));
        let l = layout(100, 30);
        let m = |x, y| {
            Event::Mouse(MouseEvent {
                kind: MouseKind::Down(MouseButton::Left),
                x,
                y,
                mods: Modifiers::NONE,
            })
        };
        a.on_event(m(3, rail_item_y(l.rail, 2)));
        assert_eq!(a.section, Section::Processes);
        a.on_event(m(l.content.x + 5, l.content.y + 5));
        assert_eq!(a.focus, Focus::Content);
    }

    #[test]
    fn file_browser_enters_and_leaves_directories() {
        let d = std::env::temp_dir().join(format!("tuidesk-app-{}", std::process::id()));
        std::fs::create_dir_all(d.join("sub")).unwrap();
        std::fs::write(d.join("sub").join("f.txt"), b"x").unwrap();
        let mut f = Files::open(d.clone());
        assert_eq!(f.entries[0].name, "sub");
        f.enter();
        assert_eq!(f.cwd, d.join("sub"));
        assert_eq!(f.entries[0].name, "f.txt");
        f.up();
        assert_eq!(f.cwd, d);
        assert_eq!(f.sel, 0, "reselecciona la carpeta de la que vienes");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn resize_is_handled_and_every_section_draws_at_any_size() {
        let mut a = App::new((80, 24));
        for (w, h) in [
            (0, 0),
            (1, 1),
            (10, 3),
            (47, 11),
            (48, 12),
            (80, 24),
            (120, 40),
            (200, 60),
        ] {
            a.on_event(Event::Resize {
                width: w,
                height: h,
            });
            for (s, _) in SECTIONS {
                a.section = s;
                a.tick();
                a.draw(&mut Buffer::new(w, h)); // no debe entrar en pánico
            }
        }
    }

    #[test]
    fn home_shows_brand_and_too_small_message() {
        let a = App::new((100, 30));
        let mut b = Buffer::new(100, 30);
        a.draw(&mut b);
        let t = text_of(&b);
        assert!(t.contains("TUI-OS") && t.contains("INICIO") && t.contains("Linux debajo"));
        let mut small = Buffer::new(30, 8);
        a.draw(&mut small);
        assert!(text_of(&small).contains("demasiado pequeño"));
    }
}
