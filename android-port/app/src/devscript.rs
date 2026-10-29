//! Developer aid: replays scripted mouse input (`HK_SCRIPT=build`) so drag and
//! drop, trimming and effects can be checked without a human at the mouse.
//! Coordinates are logical points in the default 1440x900 window.

use egui::{Event, Modifiers, PointerButton, Pos2, pos2};

pub struct DevScript {
    pub frame: u32,
    pos: Pos2,
    /// "build": media, effect and trim; "text": text preset and global library drops.
    name: String,
}

impl DevScript {
    pub fn from_env() -> Option<DevScript> {
        let name = std::env::var("HK_SCRIPT").ok()?;
        matches!(name.as_str(), "build" | "text" | "select").then_some(DevScript {
            frame: 0,
            pos: pos2(700.0, 450.0),
            name,
        })
    }

    fn mv(&mut self, to: Pos2, out: &mut Vec<Event>) {
        self.pos = to;
        out.push(Event::PointerMoved(to));
    }

    fn button(&mut self, pressed: bool, out: &mut Vec<Event>) {
        out.push(Event::PointerButton {
            pos: self.pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }

    /// One drag gesture spread over frames `start..start + steps + 3`.
    fn drag(&mut self, from: Pos2, to: Pos2, start: u32, steps: u32, out: &mut Vec<Event>) {
        let f = self.frame;
        if f == start {
            self.mv(from, out);
        } else if f == start + 1 {
            self.button(true, out);
        } else if f > start + 1 && f <= start + 1 + steps {
            let t = (f - start - 1) as f32 / steps as f32;
            self.mv(from + (to - from) * t, out);
        } else if f == start + steps + 2 {
            self.button(false, out);
        }
    }

    fn click(&mut self, at: Pos2, start: u32, out: &mut Vec<Event>) {
        let f = self.frame;
        if f == start {
            self.mv(at, out);
        } else if f == start + 1 {
            self.button(true, out);
        } else if f == start + 2 {
            self.button(false, out);
        }
    }

    pub fn events(&mut self) -> Vec<Event> {
        self.frame += 1;
        let mut out = Vec::new();
        if self.name == "select" {
            // rubber band over the text clip and the first video clips, then move them all together
            self.drag(pos2(1320.0, 704.0), pos2(185.0, 800.0), 15, 10, &mut out);
            self.drag(pos2(1000.0, 764.0), pos2(1070.0, 764.0), 45, 8, &mut out);
            return out;
        }
        if self.name == "text" {
            // text preset onto the Video 2 lane, then a global library clip onto Video 1
            self.click(pos2(158.0, 132.0), 10, &mut out);
            self.drag(pos2(100.0, 300.0), pos2(640.0, 702.0), 20, 8, &mut out);
            self.click(pos2(68.0, 132.0), 45, &mut out);
            self.drag(pos2(100.0, 355.0), pos2(1300.0, 764.0), 60, 10, &mut out);
            return out;
        }
        let lane = 757.0;
        // 1) two clips from the pool onto the video lane; the second lands near the first one's end
        self.drag(pos2(100.0, 300.0), pos2(200.0, lane), 20, 8, &mut out);
        self.drag(pos2(260.0, 300.0), pos2(536.0, lane), 40, 8, &mut out);
        // 2) a cat effect onto the cut between them
        self.click(pos2(198.0, 132.0), 60, &mut out);
        self.drag(pos2(100.0, 285.0), pos2(528.0, lane), 70, 8, &mut out);
        // 3) trim the right edge of the second clip back by about a second
        self.drag(pos2(896.0, lane), pos2(830.0, lane), 90, 6, &mut out);
        // 4) scrub to the cut so the preview shows the blend
        self.click(pos2(553.0, 700.0), 110, &mut out);
        out
    }
}
