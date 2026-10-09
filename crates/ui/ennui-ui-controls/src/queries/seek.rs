use crate::data::{Offer, Ranked};
use crate::theme::{
    COUNT_OF, ELLIPSIS, FUZZY_ASIDE, FUZZY_GAP, FUZZY_LEAD_MOST, FUZZY_MATCH, FUZZY_RUN,
    FUZZY_START, FUZZY_WORD, PALETTE_MOST, PALETTE_RECENT_MOST, PALETTE_SECTION_MOST,
    RECENT_HEADING,
};

type Scored = (usize, i32, Vec<usize>);

fn starts_word(previous: char, letter: char) -> bool {
    !previous.is_alphanumeric()
        || (previous.is_lowercase() && letter.is_uppercase())
        || (previous.is_alphabetic() && letter.is_numeric())
}

pub(crate) fn fuzzy(sought: &[char], text: &str) -> Option<(i32, Vec<usize>)> {
    let letters: Vec<char> = text.chars().collect();
    let (count, wanted) = (letters.len(), sought.len());
    if wanted == 0 {
        return Some((0, Vec::new()));
    }
    if wanted > count {
        return None;
    }
    let lower: Vec<char> = letters
        .iter()
        .map(|letter| letter.to_lowercase().next().unwrap_or(*letter))
        .collect();
    let bonus: Vec<i32> = (0..count)
        .map(|index| match index {
            0 => FUZZY_START,
            _ if starts_word(letters[index - 1], letters[index]) => FUZZY_WORD,
            _ => 0,
        })
        .collect();
    let none = i32::MIN / 2;
    let mut score = vec![none; wanted * count];
    let mut from = vec![usize::MAX; wanted * count];
    for (index, letter) in lower.iter().enumerate() {
        if *letter == sought[0] {
            let lead = (index as i32 * FUZZY_GAP).min(FUZZY_LEAD_MOST);
            score[index] = FUZZY_MATCH + bonus[index] - lead;
        }
    }
    for (step, sought_letter) in sought.iter().enumerate().skip(1) {
        let (above, here) = ((step - 1) * count, step * count);
        let mut running = (none, usize::MAX);
        for index in 0..count {
            if index >= 2 {
                let back = index - 2;
                let held = score[above + back];
                if held > none && held + FUZZY_GAP * back as i32 > running.0 {
                    running = (held + FUZZY_GAP * back as i32, back);
                }
            }
            if lower[index] != *sought_letter {
                continue;
            }
            let mut best = (none, usize::MAX);
            if index >= 1 && score[above + index - 1] > none {
                best = (score[above + index - 1] + FUZZY_RUN, index - 1);
            }
            let gapped = running.0 - FUZZY_GAP * (index as i32 - 1);
            if running.0 > none && gapped > best.0 {
                best = (gapped, running.1);
            }
            if best.0 > none {
                score[here + index] = best.0 + FUZZY_MATCH + bonus[index];
                from[here + index] = best.1;
            }
        }
    }
    let last = (wanted - 1) * count;
    let (end, total) = (0..count)
        .map(|index| (index, score[last + index]))
        .filter(|(_, held)| *held > none)
        .max_by_key(|(_, held)| *held)?;
    let mut marks = vec![0; wanted];
    let mut at = end;
    for step in (0..wanted).rev() {
        marks[step] = at;
        if step > 0 {
            at = from[step * count + at];
        }
    }
    Some((total, marks))
}

pub(crate) fn scope_of<'text>(
    scopes: &[(char, String)],
    text: &'text str,
) -> (Option<char>, &'text str) {
    let mut letters = text.chars();
    match letters.next() {
        Some(mark) if scopes.iter().any(|(held, _)| *held == mark) => {
            (Some(mark), letters.as_str())
        }
        _ => (None, text),
    }
}

pub(crate) fn switched(scopes: &[(char, String)], text: &str, back: bool) -> String {
    let (scope, rest) = scope_of(scopes, text);
    let marks: Vec<Option<char>> = std::iter::once(None)
        .chain(scopes.iter().map(|(mark, _)| Some(*mark)))
        .collect();
    let at = marks.iter().position(|held| *held == scope).unwrap_or(0);
    let next = match back {
        true => (at + marks.len() - 1) % marks.len(),
        false => (at + 1) % marks.len(),
    };
    let rest = rest.trim_start();
    match marks[next] {
        Some(mark) => format!("{mark}{rest}"),
        None => String::from(rest),
    }
}

fn section(lines: &mut Vec<Ranked>, name: &str, rows: Vec<(usize, Vec<usize>)>, most: usize) {
    if rows.is_empty() {
        return;
    }
    let count = match rows.len() > most {
        true => format!("{most} {COUNT_OF} {}", rows.len()),
        false => rows.len().to_string(),
    };
    lines.push(Ranked::Heading(String::from(name), count));
    lines.extend(
        rows.into_iter()
            .take(most)
            .map(|(place, marks)| Ranked::Row(place, marks)),
    );
}

fn scored_of(offers: &[Offer], sought: &[char], place: usize) -> Option<Scored> {
    let offer = &offers[place];
    fuzzy(sought, &offer.title)
        .map(|(score, marks)| (place, score, marks))
        .or_else(|| {
            [&offer.detail, &offer.hint]
                .into_iter()
                .filter_map(|text| fuzzy(sought, text))
                .map(|(score, _)| (place, score - FUZZY_ASIDE, Vec::new()))
                .max_by_key(|(_, score, _)| *score)
        })
}

pub(crate) fn rank(
    offers: &[Offer],
    scopes: &[(char, String)],
    text: &str,
    recent: &[u64],
) -> Vec<Ranked> {
    let (scope, rest) = scope_of(scopes, text);
    let sought: Vec<char> = rest
        .chars()
        .filter(|letter| !letter.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();
    let most = match scope {
        Some(_) => PALETTE_MOST,
        None => PALETTE_SECTION_MOST,
    };
    let pool: Vec<usize> = (0..offers.len())
        .filter(|place| scope.is_none_or(|mark| offers[*place].scope == mark))
        .collect();
    let chosen = scopes
        .iter()
        .enumerate()
        .filter(|(_, (mark, _))| scope.is_none_or(|held| held == *mark));
    let mut lines = Vec::new();
    if sought.is_empty() {
        let remembered: Vec<usize> = recent
            .iter()
            .filter_map(|payload| {
                pool.iter()
                    .copied()
                    .find(|place| offers[*place].payload == *payload)
            })
            .collect();
        let rows = remembered
            .iter()
            .map(|place| (*place, Vec::new()))
            .collect();
        section(&mut lines, RECENT_HEADING, rows, PALETTE_RECENT_MOST);
        for (_, (mark, name)) in chosen {
            let rows = pool
                .iter()
                .copied()
                .filter(|place| offers[*place].scope == *mark && !remembered.contains(place))
                .map(|place| (place, Vec::new()))
                .collect();
            section(&mut lines, name, rows, most);
        }
        return lines;
    }
    let scored: Vec<Scored> = pool
        .iter()
        .filter_map(|place| scored_of(offers, &sought, *place))
        .collect();
    let mut groups: Vec<(i32, usize, &str, Vec<Scored>)> = chosen
        .map(|(order, (mark, name))| {
            let mut rows: Vec<Scored> = scored
                .iter()
                .filter(|(place, _, _)| offers[*place].scope == *mark)
                .cloned()
                .collect();
            rows.sort_by(|first, second| {
                second
                    .1
                    .cmp(&first.1)
                    .then(
                        offers[first.0]
                            .title
                            .len()
                            .cmp(&offers[second.0].title.len()),
                    )
                    .then(first.0.cmp(&second.0))
            });
            let best = rows.first().map_or(i32::MIN, |row| row.1);
            (best, order, name.as_str(), rows)
        })
        .filter(|(_, _, _, rows)| !rows.is_empty())
        .collect();
    groups.sort_by(|first, second| second.0.cmp(&first.0).then(first.1.cmp(&second.1)));
    for (_, _, name, rows) in groups {
        let rows = rows
            .into_iter()
            .map(|(place, _, marks)| (place, marks))
            .collect();
        section(&mut lines, name, rows, most);
    }
    lines
}

pub(crate) fn first_row(lines: &[Ranked]) -> usize {
    lines
        .iter()
        .position(|line| matches!(line, Ranked::Row(..)))
        .unwrap_or(0)
}

pub(crate) fn stepped(lines: &[Ranked], lit: usize, step: i32) -> usize {
    let Some(last) = lines.len().checked_sub(1) else {
        return lit;
    };
    let wanted = (lit as i32 + step).clamp(0, last as i32) as usize;
    let row = |place: &usize| matches!(lines[*place], Ranked::Row(..));
    match step > 0 {
        true => (wanted..=last)
            .find(row)
            .or_else(|| (0..=wanted).rev().find(row)),
        false => (0..=wanted)
            .rev()
            .find(row)
            .or_else(|| (wanted..=last).find(row)),
    }
    .unwrap_or(lit)
}

pub(crate) fn offer_at<'held>(
    lines: &[Ranked],
    offers: &'held [Offer],
    place: usize,
) -> Option<&'held Offer> {
    match lines.get(place) {
        Some(Ranked::Row(at, _)) => offers.get(*at),
        _ => None,
    }
}

pub(crate) fn runs_of(text: &str, marks: &[usize], most: usize) -> Vec<(String, bool)> {
    let shown = shortened(text, most);
    let kept = match shown == text {
        true => usize::MAX,
        false => most.saturating_sub(ELLIPSIS.chars().count()),
    };
    let mut runs: Vec<(String, bool)> = Vec::new();
    for (index, letter) in shown.chars().enumerate() {
        let lit = index < kept && marks.contains(&index);
        match runs.last_mut() {
            Some((held, was)) if *was == lit => held.push(letter),
            _ => runs.push((String::from(letter), lit)),
        }
    }
    for index in 1..runs.len() {
        let kept = runs[index - 1].0.trim_end().len();
        let spaces = runs[index - 1].0.split_off(kept);
        runs[index].0.insert_str(0, &spaces);
    }
    runs.retain(|(held, _)| !held.trim().is_empty());
    runs
}

pub(crate) fn shortened(text: &str, most: usize) -> String {
    match text.chars().count() > most {
        true => format!(
            "{}{ELLIPSIS}",
            text.chars()
                .take(most.saturating_sub(ELLIPSIS.chars().count()))
                .collect::<String>()
        ),
        false => String::from(text),
    }
}
