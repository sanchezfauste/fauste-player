//! The keyboard on the playlist of the active player (itr06 I24): Up and
//! Down move the selection, Enter sets the selected entry as Next, and a
//! second Enter on the same entry within the double-click delay plays.

use egui::{Context, Key, Modifiers};
use fp_model::{AppState, Command, EntryId, PlayerId};

use super::app::{FollowScroll, ViewState};
use super::controller::Controller;

/// The Enter that armed a Play: the entry and the time of the press.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Armed {
    player: PlayerId,
    entry: EntryId,
    /// The entry on air when it was armed: a change since (the track ended
    /// or another started) makes the next Enter a first one.
    current: Option<EntryId>,
    at: f64,
}

/// Acts on the playlist keys of this frame for the active player.
pub(crate) fn handle(ctx: &Context, view: &mut ViewState, state: &AppState, ctl: &dyn Controller) {
    // A focused widget (a slider, a button) keeps the arrows and Enter, and
    // so does an open menu or combo. The non-modal Loudness and CUE windows
    // let the keys through.
    let focused = ctx.memory(|m| m.focused().is_some()) || egui::Popup::is_any_open(ctx);
    let now = ctx.input(|i| i.time);
    // Any other key or a click between two Enters breaks the pair.
    let (other_key, clicked, enter) = ctx.input(|i| {
        let mut other = false;
        let mut enter = false;
        for e in &i.events {
            if let egui::Event::Key {
                key,
                pressed: true,
                repeat,
                modifiers,
                ..
            } = e
            {
                if *key == Key::Enter && modifiers.is_none() {
                    enter |= !*repeat;
                } else {
                    other = true;
                }
            }
        }
        (other, i.pointer.any_pressed(), enter)
    });
    if other_key || clicked {
        view.enter_armed = None;
    }
    let shown = state.layout.shown_players(&state.player_ids());
    // Enter acts only on a shown active player; the arrows fall back to the
    // first shown player.
    let active = view.active_player.filter(|p| shown.contains(p));
    let Some(player) = active.or_else(|| shown.first().copied()) else {
        return;
    };
    let Some((entries, next, current)) = state.player(player).ok().and_then(|p| {
        let list = state.playlists.get(p.playlist)?;
        Some((
            list.entries.iter().map(|e| e.id).collect::<Vec<_>>(),
            p.next,
            p.current,
        ))
    }) else {
        return;
    };
    if focused {
        return;
    }
    // Each press (and repeat) moves one row.
    let (up, down) = ctx.input_mut(|i| {
        (
            i.count_and_consume_key(Modifiers::NONE, Key::ArrowUp),
            i.count_and_consume_key(Modifiers::NONE, Key::ArrowDown),
        )
    });
    if up + down > 0 {
        // The arrows moved the focus before this ran: keep it where it was.
        ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        let at = view
            .selection
            .get(&player)
            .and_then(|s| entries.iter().position(|e| e == s));
        let last = entries.len().saturating_sub(1);
        let target = match at {
            Some(i) => i
                .saturating_add_signed(down as isize - up as isize)
                .min(last),
            None => next
                .and_then(|n| entries.iter().position(|e| *e == n))
                .unwrap_or(0),
        };
        if let Some(entry) = entries.get(target).copied() {
            view.selection.insert(player, entry);
            view.active_player = Some(player);
            view.enter_armed = None;
            // Scroll the row into view: to the nearest edge, or the middle
            // when the table has not been drawn yet.
            let pass = ctx.cumulative_pass_nr();
            let align = match view.table_scroll.get(&player).and_then(|s| s.current(pass)) {
                Some(scroll) => scroll.reveal_align(target),
                None => Some(egui::Align::Center),
            };
            if let Some(align) = align {
                view.follow_scroll.insert(
                    player,
                    FollowScroll {
                        entry,
                        align,
                        animated: false,
                    },
                );
            }
        }
    }
    let selected = view
        .selection
        .get(&player)
        .copied()
        .filter(|s| entries.contains(s));
    let Some(entry) = selected.filter(|_| active == Some(player)) else {
        return;
    };
    if !enter {
        return;
    }
    // Take the press (and its repeats) so nothing else acts on it.
    ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
    let delay = ctx.options(|o| o.input_options.max_double_click_delay);
    let play = view.enter_armed.is_some_and(|a| {
        a.player == player
            && a.entry == entry
            && a.current == current
            && next == Some(entry)
            && now - a.at <= delay
    });
    if play {
        view.enter_armed = None;
        send(ctl, state, Command::Play(player));
    } else {
        view.enter_armed = Some(Armed {
            player,
            entry,
            current,
            at: now,
        });
        send(ctl, state, Command::SetNext(player, entry));
    }
}

fn send(ctl: &dyn Controller, state: &AppState, command: Command) {
    if fp_model::command_available(state, &command) {
        ctl.send(command);
    }
}
