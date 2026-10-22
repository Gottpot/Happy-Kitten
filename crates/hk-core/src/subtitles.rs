//! Subtitle and lyric files: SRT, LRC and plain text (one line per cue).

use crate::time::Tick;

#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub start: Tick,
    pub end: Tick,
    pub text: String,
}

fn clean(input: &str) -> String {
    input
        .trim_start_matches('\u{feff}')
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

/// `00:01:02,500` or `00:01:02.500` or `01:02,5` to seconds.
fn parse_clock(s: &str) -> Option<f64> {
    let s = s.trim().replace(',', ".");
    let parts: Vec<&str> = s.split(':').collect();
    let nums: Option<Vec<f64>> = parts.iter().map(|p| p.trim().parse::<f64>().ok()).collect();
    let nums = nums?;
    let secs = match nums.as_slice() {
        [h, m, s] => h * 3600.0 + m * 60.0 + s,
        [m, s] => m * 60.0 + s,
        [s] => *s,
        _ => return None,
    };
    (secs.is_finite() && secs >= 0.0).then_some(secs)
}

pub fn parse_srt(input: &str) -> Vec<Cue> {
    let text = clean(input);
    let mut cues = Vec::new();
    for block in text.split("\n\n") {
        let mut lines = block
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.trim().is_empty());
        let mut first = lines.next();
        // optional numeric index line
        if first.is_some_and(|l| l.trim().chars().all(|c| c.is_ascii_digit())) {
            first = lines.next();
        }
        let Some(timing) = first else { continue };
        let Some((a, b)) = timing.split_once("-->") else {
            continue;
        };
        // trailing position info ("X1:..") may follow the end time
        let b = b.split_whitespace().next().unwrap_or("");
        let (Some(start), Some(end)) = (parse_clock(a), parse_clock(b)) else {
            continue;
        };
        let body: Vec<&str> = lines.collect();
        let body = strip_tags(&body.join("\n"));
        if end > start && !body.trim().is_empty() {
            cues.push(Cue {
                start: Tick::from_secs(start),
                end: Tick::from_secs(end),
                text: body.trim().to_string(),
            });
        }
    }
    cues
}

/// Remove `<i>`, `<b>`, `{\an8}` style markup.
fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let (mut in_angle, mut in_brace) = (false, false);
    for c in s.chars() {
        match c {
            '<' => in_angle = true,
            '>' if in_angle => in_angle = false,
            '{' => in_brace = true,
            '}' if in_brace => in_brace = false,
            _ if in_angle || in_brace => {}
            _ => out.push(c),
        }
    }
    out
}

/// `[mm:ss.xx] words`; each line lasts until the next one (at most `max_line` seconds).
pub fn parse_lrc(input: &str, max_line: f64) -> Vec<Cue> {
    let text = clean(input);
    let mut stamped: Vec<(f64, String)> = Vec::new();
    for line in text.lines() {
        let mut rest = line.trim();
        let mut times = Vec::new();
        while let Some(stripped) = rest.strip_prefix('[') {
            let Some((tag, after)) = stripped.split_once(']') else {
                break;
            };
            match parse_clock(tag) {
                Some(t)
                    if tag.contains(':')
                        && tag.chars().next().is_some_and(|c| c.is_ascii_digit()) =>
                {
                    times.push(t)
                }
                _ => {
                    times.clear();
                    rest = "";
                    break;
                }
            }
            rest = after.trim_start();
        }
        for t in times {
            if !rest.trim().is_empty() {
                stamped.push((t, rest.trim().to_string()));
            }
        }
    }
    stamped.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut cues = Vec::new();
    for (i, (t, words)) in stamped.iter().enumerate() {
        let next = stamped.get(i + 1).map(|n| n.0);
        let end = match next {
            Some(n) => n.min(t + max_line),
            None => t + max_line.min(5.0),
        };
        if end > *t {
            cues.push(Cue {
                start: Tick::from_secs(*t),
                end: Tick::from_secs(end),
                text: words.clone(),
            });
        }
    }
    cues
}

/// One cue per non-empty line, each `secs_per_line` long, back to back from `offset`.
pub fn parse_plain(input: &str, secs_per_line: f64, offset: Tick) -> Vec<Cue> {
    let mut at = offset;
    clean(input)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| {
            let cue = Cue {
                start: at,
                end: at + Tick::from_secs(secs_per_line),
                text: l.to_string(),
            };
            at = cue.end;
            cue
        })
        .collect()
}

/// Pick the right parser by file extension (`srt`, `lrc`, anything else = plain lines).
pub fn parse_by_extension(ext: &str, input: &str, offset: Tick) -> Vec<Cue> {
    match ext.to_ascii_lowercase().as_str() {
        "srt" => parse_srt(input),
        "lrc" => parse_lrc(input, 10.0),
        _ => parse_plain(input, 3.0, offset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srt_with_bom_crlf_tags_and_two_lines() {
        let srt = "\u{feff}1\r\n00:00:01,000 --> 00:00:03,500\r\n<i>Hallo</i> Welt\r\nzweite Zeile\r\n\r\n2\r\n00:00:04,000 --> 00:00:05,000 X1:0\r\nNoch eins\r\n";
        let cues = parse_srt(srt);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].start, Tick::from_secs(1.0));
        assert_eq!(cues[0].end, Tick::from_secs(3.5));
        assert_eq!(cues[0].text, "Hallo Welt\nzweite Zeile");
        assert_eq!(cues[1].text, "Noch eins");
    }

    #[test]
    fn srt_garbage_gives_nothing() {
        assert!(parse_srt("das ist kein srt").is_empty());
        assert!(parse_srt("1\n00:00:05,000 --> 00:00:01,000\nverkehrt").is_empty());
        assert!(parse_srt("").is_empty());
    }

    #[test]
    fn lrc_lines_last_until_the_next() {
        let lrc = "[ar:Katze]\n[00:10.00]Miau\n[00:12.50]Schnurr\n[00:20.00]Ende";
        let cues = parse_lrc(lrc, 10.0);
        assert_eq!(cues.len(), 3);
        assert_eq!(cues[0].end, Tick::from_secs(12.5));
        assert_eq!(cues[1].end, Tick::from_secs(20.0));
        assert_eq!(cues[2].end, Tick::from_secs(25.0));
        assert_eq!(cues[0].text, "Miau");
    }

    #[test]
    fn lrc_with_several_time_tags_repeats_the_line() {
        let cues = parse_lrc("[00:01.00][00:05.00]Refrain", 10.0);
        assert_eq!(cues.len(), 2);
    }

    #[test]
    fn plain_lines_follow_each_other() {
        let cues = parse_plain("a\n\n b \nc", 2.0, Tick::from_secs(1.0));
        assert_eq!(cues.len(), 3);
        assert_eq!(cues[1].start, Tick::from_secs(3.0));
        assert_eq!(cues[1].text, "b");
    }
}
